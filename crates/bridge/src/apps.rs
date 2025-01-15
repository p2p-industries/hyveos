use hyveos_core::{
    apps::RunningApp,
    grpc::{self, apps_server::Apps},
};
use libp2p::PeerId;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use ulid::Ulid;

use crate::{Telemetry, TonicResult};

#[trait_variant::make(Send)]
pub trait AppsClient: Sync + 'static {
    type Error: ToString;

    async fn deploy_image(
        &self,
        image: &str,
        local: bool,
        peer_id: PeerId,
        verbose: bool,
        ports: impl IntoIterator<Item = u16> + Send,
        persistent: bool,
    ) -> Result<Ulid, Self::Error>;

    async fn self_deploy_image(
        &self,
        image: &str,
        local: bool,
        verbose: bool,
        ports: impl IntoIterator<Item = u16> + Send,
        persistent: bool,
    ) -> Result<Ulid, Self::Error>;

    async fn list_containers(
        &self,
        peer_id: Option<PeerId>,
    ) -> Result<Vec<RunningApp>, Self::Error>;

    async fn stop_container(
        &self,
        container_id: Ulid,
        peer_id: Option<PeerId>,
    ) -> Result<(), Self::Error>;
}

pub struct AppsServer<C> {
    client: C,
    ulid: Option<Ulid>,
    telemetry: Telemetry,
}

impl<C: AppsClient> AppsServer<C> {
    pub fn new(client: C, ulid: Option<Ulid>, telemetry: Telemetry) -> Self {
        Self {
            client,
            ulid,
            telemetry,
        }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl<C: AppsClient> Apps for AppsServer<C> {
    async fn deploy(&self, request: TonicRequest<grpc::DeployAppRequest>) -> TonicResult<grpc::Id> {
        self.telemetry.track("apps.deploy");
        let request = request.into_inner();

        tracing::debug!(?request, "Received deploy request");

        let grpc::DeployAppRequest {
            app:
                grpc::DockerApp {
                    image: grpc::DockerImage { name },
                    ports,
                },
            local,
            peer,
            persistent,
        } = request;

        let ports = ports
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::invalid_argument(format!("Invalid port number: {e}")))?;

        let id = if let Some(peer_id) = peer.map(TryInto::try_into).transpose()? {
            self.client
                .deploy_image(&name, local, peer_id, false, ports, persistent)
                .await
        } else {
            self.client
                .self_deploy_image(&name, local, false, ports, persistent)
                .await
        }
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(id.into()))
    }

    async fn list_running(
        &self,
        request: TonicRequest<grpc::ListRunningAppsRequest>,
    ) -> TonicResult<grpc::RunningApps> {
        self.telemetry.track("apps.list_running");
        let request = request.into_inner();

        tracing::debug!(?request, "Received list_running request");

        let peer_id = request.peer.map(TryInto::try_into).transpose()?;

        self.client
            .list_containers(peer_id)
            .await
            .map(|containers| {
                let apps = containers.into_iter().map(Into::into).collect();

                TonicResponse::new(grpc::RunningApps { apps })
            })
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn stop(&self, request: TonicRequest<grpc::StopAppRequest>) -> TonicResult<grpc::Empty> {
        self.telemetry.track("apps.stop");
        let request = request.into_inner();

        tracing::debug!(?request, "Received stop request");

        let grpc::StopAppRequest { id, peer } = request;

        let peer_id = peer.map(TryInto::try_into).transpose()?;

        self.client
            .stop_container(id.try_into()?, peer_id)
            .await
            .map(|()| TonicResponse::new(grpc::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_own_app_id(&self, _: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Id> {
        self.telemetry.track("apps.get_own_app_id");
        self.ulid
            .map(Into::into)
            .map(TonicResponse::new)
            .ok_or(Status::internal("Can only get ID of application bridge"))
    }
}
