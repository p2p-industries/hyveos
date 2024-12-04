#[cfg(feature = "batman")]
use futures::stream::TryStreamExt as _;
#[cfg(feature = "batman")]
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::grpc::{self, discovery_server::Discovery};
use p2p_stack::Client;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, TonicResult};

pub struct DiscoveryServer {
    client: Client,
}

impl DiscoveryServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Discovery for DiscoveryServer {
    type SubscribeEventsStream = ServerStream<grpc::NeighbourEvent>;

    #[cfg(feature = "batman")]
    async fn subscribe_events(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeEventsStream> {
        tracing::debug!("Received subscribe_events request");

        let stream = self
            .client
            .neighbours()
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?
            .map_ok(|event| NeighbourEvent::from(event.as_ref()).into())
            .map_err(|e| Status::internal(e.to_string()));

        Ok(TonicResponse::new(Box::pin(stream)))
    }

    #[cfg(not(feature = "batman"))]
    async fn subscribe_events(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeEventsStream> {
        return Err(Status::unavailable("batman feature is not enabled"));
    }

    async fn get_own_id(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Peer> {
        tracing::debug!("Received get_own_id request");

        Ok(TonicResponse::new(self.client.peer_id().into()))
    }
}
