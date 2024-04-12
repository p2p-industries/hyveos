use libp2p::gossipsub::{IdentTopic, Message, MessageId, PublishError, SubscriptionError};
use tokio::sync::{broadcast, mpsc, oneshot};

use super::actor::ReceivedMessage;

pub enum Command {
    PublishMessage {
        topic: IdentTopic,
        data: Vec<u8>,
        send_message_id: oneshot::Sender<Result<MessageId, PublishError>>,
    },
    Subscribe {
        topic: IdentTopic,
        send_subscription:
            oneshot::Sender<Result<broadcast::Receiver<ReceivedMessage>, SubscriptionError>>,
    },
}

impl From<Command> for crate::p2p::command::Command {
    fn from(command: Command) -> Self {
        crate::p2p::command::Command::Gossipsub(command)
    }
}
