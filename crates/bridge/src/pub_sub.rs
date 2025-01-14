use futures::stream::{StreamExt as _, TryStreamExt as _};
use hyveos_core::grpc::{self, pub_sub_server::PubSub};
use hyveos_p2p_stack::Client;
use libp2p::gossipsub::IdentTopic;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, Telemetry, TonicResult};

pub struct PubSubServer {
    client: Client,
    telemetry: Telemetry,
}

impl PubSubServer {
    pub fn new(client: Client, telemetry: Telemetry) -> Self {
        Self { client, telemetry }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl PubSub for PubSubServer {
    type SubscribeStream = ServerStream<grpc::PubSubRecvMessage>;

    async fn subscribe(
        &self,
        request: TonicRequest<grpc::Topic>,
    ) -> TonicResult<Self::SubscribeStream> {
        self.telemetry.track("pub_sub.subscribe");
        let request = request.into_inner();

        tracing::debug!(?request, "Received subscribe request");

        let topic = request.topic;

        let receiver = self
            .client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("app/{topic}")))
            .subscribe()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(Into::into)
            .map_err(|e| Status::internal(e.to_string()))
            .boxed();

        Ok(TonicResponse::new(stream))
    }

    async fn publish(
        &self,
        request: TonicRequest<grpc::PubSubMessage>,
    ) -> TonicResult<grpc::PubSubMessageId> {
        self.telemetry.track("pub_sub.publish");
        let request = request.into_inner();

        tracing::debug!(?request, "Received publish request");

        let grpc::PubSubMessage {
            data,
            topic: grpc::Topic { topic },
        } = request;

        self.client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("app/{topic}")))
            .publish(data.into())
            .await
            .map(Into::into)
            .map(TonicResponse::new)
            .map_err(|e| Status::internal(e.to_string()))
    }
}
