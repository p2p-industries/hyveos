use libp2p::gossipsub::{IdentTopic, PublishError, SubscriptionError};
use p2p_industries_core::{
    debug::MessageDebugEventType,
    gossipsub::{MessageId, ReceivedMessage},
};
use tokio::sync::{broadcast, oneshot};

use super::Command;
use crate::client::{RequestError, SpecialClient};

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

    pub async fn debug_subscribe(
        &self,
    ) -> Result<broadcast::Receiver<MessageDebugEventType>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::DebugSubscribe(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
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
