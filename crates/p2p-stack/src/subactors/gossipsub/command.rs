use libp2p::gossipsub::{IdentTopic, PublishError, SubscriptionError};
use p2p_industries_core::gossipsub::{MessageId, ReceivedMessage};
use tokio::sync::{broadcast, oneshot};

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
