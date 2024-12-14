use hyveos_core::{
    grpc::{self, scripting_server::Scripting},
    scripting::RunningScript,
};
use libp2p::PeerId;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use ulid::Ulid;

use crate::TonicResult;

#[trait_variant::make(Send)]
pub trait ScriptingClient: Sync + 'static {
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
    ) -> Result<Vec<RunningScript>, Self::Error>;

    async fn stop_container(
        &self,
        container_id: Ulid,
        peer_id: Option<PeerId>,
    ) -> Result<(), Self::Error>;
}

pub struct ScriptingServer<C> {
    client: C,
    ulid: Option<Ulid>,
}

impl<C: ScriptingClient> ScriptingServer<C> {
    pub fn new(client: C, ulid: Option<Ulid>) -> Self {
        Self { client, ulid }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl<C: ScriptingClient> Scripting for ScriptingServer<C> {
    async fn deploy_script(
        &self,
        request: TonicRequest<grpc::DeployScriptRequest>,
    ) -> TonicResult<grpc::Id> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received deploy_script request");

        let grpc::DeployScriptRequest {
            script:
                grpc::DockerScript {
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

    async fn list_running_scripts(
        &self,
        request: TonicRequest<grpc::ListRunningScriptsRequest>,
    ) -> TonicResult<grpc::RunningScripts> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received list_running_scripts request");

        let peer_id = request.peer.map(TryInto::try_into).transpose()?;

        self.client
            .list_containers(peer_id)
            .await
            .map(|containers| {
                let scripts = containers.into_iter().map(Into::into).collect();

                TonicResponse::new(grpc::RunningScripts { scripts })
            })
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn stop_script(
        &self,
        request: TonicRequest<grpc::StopScriptRequest>,
    ) -> TonicResult<grpc::Empty> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received stop_script request");

        let grpc::StopScriptRequest { id, peer } = request;

        let peer_id = peer.map(TryInto::try_into).transpose()?;

        self.client
            .stop_container(id.try_into()?, peer_id)
            .await
            .map(|()| TonicResponse::new(grpc::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_own_id(&self, _: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Id> {
        self.ulid
            .map(Into::into)
            .map(TonicResponse::new)
            .ok_or(Status::internal("Can only get ID of scripting bridge"))
    }
}
