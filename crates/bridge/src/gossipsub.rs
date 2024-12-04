use futures::stream::TryStreamExt as _;
use hyveos_core::grpc::{self, gossip_sub_server::GossipSub};
use libp2p::gossipsub::IdentTopic;
use p2p_stack::Client;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, TonicResult};

pub struct GossipSubServer {
    client: Client,
}

impl GossipSubServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl GossipSub for GossipSubServer {
    type SubscribeStream = ServerStream<grpc::GossipSubRecvMessage>;

    async fn subscribe(
        &self,
        request: TonicRequest<grpc::Topic>,
    ) -> TonicResult<Self::SubscribeStream> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received subscribe request");

        let topic = request.topic;

        let receiver = self
            .client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("script/{topic}")))
            .subscribe()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(Into::into)
            .map_err(|e| Status::internal(e.to_string()));

        Ok(TonicResponse::new(Box::pin(stream)))
    }

    async fn publish(
        &self,
        request: TonicRequest<grpc::GossipSubMessage>,
    ) -> TonicResult<grpc::GossipSubMessageId> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received publish request");

        let grpc::GossipSubMessage {
            data,
            topic: grpc::Topic { topic },
        } = request;

        self.client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("script/{topic}")))
            .publish(data.into())
            .await
            .map(Into::into)
            .map(TonicResponse::new)
            .map_err(|e| Status::internal(e.to_string()))
    }
}
