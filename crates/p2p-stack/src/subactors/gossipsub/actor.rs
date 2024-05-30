use std::collections::HashMap;

use libp2p::{
    gossipsub::{
        Behaviour, Event, IdentTopic, Message, MessageId, PublishError, SubscriptionError,
        TopicHash,
    },
    PeerId,
};
use tokio::sync::broadcast;

use super::Command;
use crate::{actor::SubActor, behaviour::MyBehaviour};

const CHANNEL_CAP: usize = 10;

#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    pub propagation_source: PeerId,
    pub message_id: MessageId,
    pub message: Message,
}

#[derive(Debug, Default)]
pub struct Actor {
    topic_subscriptions: HashMap<TopicHash, (IdentTopic, broadcast::Sender<ReceivedMessage>)>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Subscription error: {0}")]
    Subscription(#[from] SubscriptionError),
    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("Send messsage id failed: `{0:?}`")]
    MessageIdFailed(Result<MessageId, PublishError>),
    #[error("Send subscription failed: `{0:?}`")]
    SubscriptionFailed(Result<broadcast::Receiver<ReceivedMessage>, SubscriptionError>),
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("Message without topic: `{0}`")]
    MessageWithoutTopic(TopicHash),
    #[error("Broadcast error: `{0}`")]
    Broadcast(Box<broadcast::error::SendError<ReceivedMessage>>),
}

impl SubActor for Actor {
    type SubCommand = Command;
    type CommandError = CommandError;
    type Event = Event;
    type EventError = EventError;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        self.garbage_collect(&mut behaviour.gossipsub)?;
        match command {
            Command::PublishMessage {
                topic,
                data,
                send_message_id,
            } => send_message_id
                .send(behaviour.gossipsub.publish(topic, data))
                .map_err(CommandError::MessageIdFailed),
            Command::Subscribe {
                topic,
                send_subscription,
            } => send_subscription
                .send(self.get_sub(&mut behaviour.gossipsub, topic))
                .map_err(CommandError::SubscriptionFailed),
        }
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::Message {
                propagation_source,
                message_id,
                message,
            } => {
                self.garbage_collect(&mut behaviour.gossipsub)?;
                let topic_hash = message.topic.clone();
                let received_message = ReceivedMessage {
                    propagation_source,
                    message_id,
                    message,
                };
                self.topic_subscriptions.get(&topic_hash).map_or_else(
                    || Err(EventError::MessageWithoutTopic(topic_hash)),
                    |(_, sender)| {
                        sender
                            .send(received_message)
                            .map_err(Box::new)
                            .map_err(EventError::Broadcast)
                            .map(|_| ())
                    },
                )
            }
            _ => Ok(()),
        }
    }
}

impl Actor {
    pub fn get_sub(
        &mut self,
        behaviour: &mut Behaviour,
        topic: IdentTopic,
    ) -> Result<broadcast::Receiver<ReceivedMessage>, SubscriptionError> {
        if let Some((_, sender)) = self.topic_subscriptions.get(&topic.hash()) {
            return Ok(sender.subscribe());
        }
        if behaviour.subscribe(&topic)? {
            tracing::info!("Subscribed newly to topic: {:?}", topic);
        } else {
            tracing::warn!(
                "Already subscribed to topic: {:?}. Inconsistency error.",
                topic
            );
        }
        let (sender, receiver) = broadcast::channel(CHANNEL_CAP);
        self.topic_subscriptions
            .insert(topic.hash(), (topic, sender));
        Ok(receiver)
    }

    pub fn garbage_collect(&mut self, behaviour: &mut Behaviour) -> Result<(), PublishError> {
        for (topic, sender) in self.topic_subscriptions.values() {
            if sender.receiver_count() == 0 {
                tracing::info!("Unsubscribing from topic: {:?}", topic);
                if !behaviour.unsubscribe(topic)? {
                    tracing::warn!("There existed a sender but no subscription on the behviour. Inconsistency error.");
                }
            }
        }
        self.topic_subscriptions
            .retain(|_, (_, sender)| sender.receiver_count() > 0);
        Ok(())
    }
}