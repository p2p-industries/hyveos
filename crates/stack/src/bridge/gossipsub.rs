use futures::stream::TryStreamExt as _;
use libp2p::gossipsub::IdentTopic;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::p2p::Client;

use super::{
    script::{self, gossip_sub_server::GossipSub},
    ServerStream, TonicResult,
};

pub struct GossipSubServer {
    client: Client,
}

impl GossipSubServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl GossipSub for GossipSubServer {
    type SubscribeStream = ServerStream<script::GossipSubRecvMessage>;

    async fn subscribe(
        &self,
        request: TonicRequest<script::Topic>,
    ) -> TonicResult<Self::SubscribeStream> {
        let topic = request.into_inner().topic;

        let receiver = self
            .client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("script/{topic}")))
            .subscribe()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(move |message| script::GossipSubRecvMessage {
                peer_id: message.propagation_source.to_string(),
                msg: script::GossipSubMessage {
                    data: message.message.data,
                    topic: script::Topic {
                        topic: topic.clone(),
                    },
                },
                msg_id: script::GossipSubMessageId {
                    id: message.message_id.0,
                },
            })
            .map_err(|e| Status::internal(e.to_string()));

        Ok(TonicResponse::new(Box::pin(stream)))
    }

    async fn publish(
        &self,
        request: TonicRequest<script::GossipSubMessage>,
    ) -> TonicResult<script::GossipSubMessageId> {
        let script::GossipSubMessage {
            data,
            topic: script::Topic { topic },
        } = request.into_inner();

        self.client
            .gossipsub()
            .get_topic(IdentTopic::new(format!("script/{topic}")))
            .publish(data)
            .await
            .map(|message_id| TonicResponse::new(script::GossipSubMessageId { id: message_id.0 }))
            .map_err(|e| Status::internal(e.to_string()))
    }
}
