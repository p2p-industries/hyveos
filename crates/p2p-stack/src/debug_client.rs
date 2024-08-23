use std::collections::HashSet;

use futures::stream::StreamExt as _;
use libp2p::{gossipsub::IdentTopic, PeerId};
use p2p_industries_core::{debug::MeshTopologyEvent, discovery, gossipsub::ReceivedMessage};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{subactors::gossipsub::TopicHandle, Client, NeighbourEvent};

const GOSSIPSUB_TOPIC: &str = "debug/neighbour_events";

#[derive(Debug, Clone, Serialize, Deserialize)]
enum GossipsubMessage {
    Subscribe(PeerId),
    Unsubscribe(PeerId),
}

pub enum Command {
    SubscribeNeighbourEvents(oneshot::Sender<broadcast::Receiver<MeshTopologyEvent>>),
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
                Some(Ok(neighbour_event)) = neighbour_events.next() => {
                    self.handle_neighbour_event(neighbour_event.as_ref()).await;
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

    async fn handle_neighbour_event(&mut self, neighbour_event: &NeighbourEvent) {
        if !self.neighbour_event_subscribers.is_empty() {
            if matches!(neighbour_event, NeighbourEvent::Init(_)) {
                return;
            }

            for subscriber in &self.neighbour_event_subscribers {
                self.client
                    .debug()
                    .send_neighbour_event(*subscriber, neighbour_event.into())
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
                        .send_neighbour_event(
                            peer_id,
                            discovery::NeighbourEvent::Init(current_neighbours),
                        )
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
