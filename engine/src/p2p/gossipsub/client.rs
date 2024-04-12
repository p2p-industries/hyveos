use libp2p::gossipsub::{IdentTopic, Message, MessageId, PublishError, SubscriptionError};
use tokio::sync::{broadcast, oneshot};

use crate::p2p::client::{RequestError, SpecialClient};

use super::{actor::ReceivedMessage, Command};

pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub fn get_topic(&self, topic: IdentTopic) -> TopicHandle {
        let commander = self.inner.clone();
        TopicHandle { commander, topic }
    }
}

pub struct TopicHandle {
    commander: SpecialClient<Command>,
    topic: IdentTopic,
}

impl TopicHandle {
    pub async fn publish(&self, data: Vec<u8>) -> Result<MessageId, RequestError<PublishError>> {
        let (sender, receiver) = oneshot::channel();
        self.commander
            .request(
                Command::PublishMessage {
                    topic: self.topic.clone(),
                    data,
                    send_message_id: sender,
                },
                receiver,
            )
            .await
    }

    pub async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<ReceivedMessage>, RequestError<SubscriptionError>> {
        let (sender, receiver) = oneshot::channel();
        self.commander
            .request(
                Command::Subscribe {
                    topic: self.topic.clone(),
                    send_subscription: sender,
                },
                receiver,
            )
            .await
    }
}
