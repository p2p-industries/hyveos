#[cfg(feature = "batman")]
use futures::stream::{StreamExt as _, TryStreamExt as _};
use hyveos_core::grpc::{self, neighbours_server::Neighbours};
#[cfg(feature = "batman")]
use hyveos_core::neighbours::NeighbourEvent;
use hyveos_p2p_stack::Client;
#[cfg(feature = "batman")]
use tonic::Response as TonicResponse;
use tonic::{Request as TonicRequest, Status};

use crate::{ServerStream, Telemetry, TonicResult};

#[cfg_attr(not(feature = "batman"), allow(dead_code))]
pub struct NeighboursServer {
    client: Client,
    telemetry: Telemetry,
}

impl NeighboursServer {
    pub fn new(client: Client, telemetry: Telemetry) -> Self {
        Self { client, telemetry }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Neighbours for NeighboursServer {
    type SubscribeStream = ServerStream<grpc::NeighbourEvent>;

    #[cfg(feature = "batman")]
    async fn subscribe(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeStream> {
        self.telemetry.track("neighbours.subscribe");
        tracing::debug!("Received subscribe request");

        let stream = self
            .client
            .neighbours()
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?
            .map_ok(|event| NeighbourEvent::from(event.as_ref()).into())
            .map_err(|e| Status::internal(e.to_string()))
            .boxed();

        Ok(TonicResponse::new(stream))
    }

    #[cfg(not(feature = "batman"))]
    async fn subscribe(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeStream> {
        return Err(Status::unavailable("batman feature is not enabled"));
    }

    #[cfg(feature = "batman")]
    async fn get(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Peers> {
        self.telemetry.track("neighbours.get");
        tracing::debug!("Received get request");

        let peers = self
            .client
            .neighbours()
            .get_resolved()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?
            .into_keys()
            .map(Into::into)
            .collect();

        Ok(TonicResponse::new(grpc::Peers { peers }))
    }

    #[cfg(not(feature = "batman"))]
    async fn get(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Peers> {
        return Err(Status::unavailable("batman feature is not enabled"));
    }
}
