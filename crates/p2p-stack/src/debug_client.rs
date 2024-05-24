use std::{collections::HashSet, sync::Arc};

use libp2p::{gossipsub::IdentTopic, PeerId};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{
    subactors::{
        debug::NeighbourEvent,
        gossipsub::{ReceivedMessage, TopicHandle},
        neighbours,
    },
    Client,
};

const GOSSIPSUB_TOPIC: &str = "debug/neighbour_events";

#[derive(Debug, Clone, Serialize, Deserialize)]
enum GossipsubMessage {
    Subscribe(PeerId),
    Unsubscribe(PeerId),
}

pub enum Command {
    SubscribeNeighbourEvents(oneshot::Sender<broadcast::Receiver<(PeerId, NeighbourEvent)>>),
    UnsubscribeNeighbourEvents,
}

pub struct DebugClient {
    client: Client,
    topic: TopicHandle,
    receiver: mpsc::Receiver<Command>,
    neighbour_event_subscribers: HashSet<PeerId>,
}

impl DebugClient {
    pub fn build(client: Client) -> (Self, mpsc::Sender<Command>) {
        let topic = client
            .gossipsub()
            .get_topic(IdentTopic::new(GOSSIPSUB_TOPIC));

        let (sender, receiver) = mpsc::channel(2);
        (
            Self {
                client,
                topic,
                receiver,
                neighbour_event_subscribers: HashSet::new(),
            },
            sender,
        )
    }

    pub async fn run(mut self) {
        let mut neighbour_events = self.client.neighbours().subscribe().await.unwrap();
        let mut gossipsub_messages = self.topic.subscribe().await.unwrap();

        loop {
            tokio::select! {
                Some(command) = self.receiver.recv() => {
                    self.handle_command(command).await;
                }
                Ok(neighbour_event) = neighbour_events.recv() => {
                    self.handle_neighbour_event(neighbour_event).await;
                }
                Ok(gossipsub_message) = gossipsub_messages.recv() => {
                    self.handle_gossipsub_message(&gossipsub_message).await;
                }
                else => {}
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        let peer_id = self.client.peer_id();

        let message = match command {
            Command::SubscribeNeighbourEvents(sender) => {
                let receiver = self
                    .client
                    .debug()
                    .subscribe_neighbour_events()
                    .await
                    .unwrap();
                sender.send(receiver).unwrap();

                GossipsubMessage::Subscribe(peer_id)
            }
            Command::UnsubscribeNeighbourEvents => GossipsubMessage::Unsubscribe(peer_id),
        };

        let message = cbor4ii::serde::to_vec(Vec::new(), &message).unwrap();
        self.topic.publish(message).await.unwrap();
    }

    async fn handle_neighbour_event(&mut self, neighbour_event: Arc<neighbours::Event>) {
        if !self.neighbour_event_subscribers.is_empty() {
            let event = match neighbour_event.as_ref() {
                neighbours::Event::ResolvedNeighbour(neighbour) => {
                    NeighbourEvent::Discovered(neighbour.peer_id)
                }
                neighbours::Event::LostNeighbour(neighbour) => {
                    NeighbourEvent::Lost(neighbour.peer_id)
                }
            };

            for subscriber in &self.neighbour_event_subscribers {
                self.client
                    .debug()
                    .send_neighbour_event(*subscriber, event.clone())
                    .await
                    .unwrap();
            }
        }
    }

    async fn handle_gossipsub_message(&mut self, message: &ReceivedMessage) {
        if message.propagation_source != self.client.peer_id() {
            match cbor4ii::serde::from_slice(&message.message.data).unwrap() {
                GossipsubMessage::Subscribe(peer_id) => {
                    let current_neighbours = self
                        .client
                        .neighbours()
                        .get_resolved()
                        .await
                        .unwrap()
                        .keys()
                        .copied()
                        .collect();
                    self.client
                        .debug()
                        .send_neighbour_event(peer_id, NeighbourEvent::Init(current_neighbours))
                        .await
                        .unwrap();

                    self.neighbour_event_subscribers.insert(peer_id);
                }
                GossipsubMessage::Unsubscribe(peer_id) => {
                    self.neighbour_event_subscribers.remove(&peer_id);
                }
            }
        }
    }
}
