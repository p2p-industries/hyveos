use libp2p::gossipsub::{IdentTopic, MessageId, PublishError, SubscriptionError};
use tokio::sync::{broadcast, oneshot};

use super::actor::ReceivedMessage;
use crate::impl_from_special_command;

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

impl_from_special_command!(Gossipsub);
