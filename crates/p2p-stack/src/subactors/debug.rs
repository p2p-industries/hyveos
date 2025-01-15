use hyveos_core::{
    debug::{MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType},
    neighbours::NeighbourEvent,
};
use libp2p::{
    request_response::{cbor, Config, Event, Message as CborMessage, ProtocolSupport},
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, oneshot};

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    impl_from_special_command,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    NeighbourEvent(NeighbourEvent),
    MessageEvent(MessageDebugEventType),
}

pub type Behaviour = cbor::Behaviour<Message, ()>;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/debug"), ProtocolSupport::Full)],
        Config::default(),
    )
}

pub enum Command {
    SubscribeNeighbourEvents(oneshot::Sender<broadcast::Receiver<MeshTopologyEvent>>),
    SendNeighbourEvent {
        peer_id: PeerId,
        event: NeighbourEvent,
    },
    SubscribeMessageEvents(oneshot::Sender<broadcast::Receiver<MessageDebugEvent>>),
    SendMessageEvent {
        peer_id: PeerId,
        event: MessageDebugEventType,
    },
}

impl_from_special_command!(Debug);

#[derive(Debug)]
pub struct Actor {
    neighbour_event_sender: broadcast::Sender<MeshTopologyEvent>,
    message_event_sender: broadcast::Sender<MessageDebugEvent>,
}

impl Default for Actor {
    fn default() -> Self {
        let (neighbour_event_sender, _) = broadcast::channel(10);
        let (message_event_sender, _) = broadcast::channel(10);
        Self {
            neighbour_event_sender,
            message_event_sender,
        }
    }
}

impl SubActor for Actor {
    type SubCommand = Command;
    type Event = <Behaviour as NetworkBehaviour>::ToSwarm;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::SubscribeNeighbourEvents(sender) => {
                sender
                    .send(self.neighbour_event_sender.subscribe())
                    .unwrap();
            }
            Command::SendNeighbourEvent { peer_id, event } => {
                behaviour
                    .debug
                    .send_request(&peer_id, Message::NeighbourEvent(event));
            }
            Command::SubscribeMessageEvents(sender) => {
                sender.send(self.message_event_sender.subscribe()).unwrap();
            }
            Command::SendMessageEvent { peer_id, event } => {
                behaviour
                    .debug
                    .send_request(&peer_id, Message::MessageEvent(event));
            }
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::Message {
                peer,
                message:
                    CborMessage::Request {
                        request: Message::NeighbourEvent(event),
                        ..
                    },
            } => {
                let _ = self.neighbour_event_sender.send(MeshTopologyEvent {
                    peer_id: peer,
                    event,
                });
            }
            Event::Message {
                peer,
                message:
                    CborMessage::Request {
                        request: Message::MessageEvent(event),
                        ..
                    },
            } => {
                let _ = self.message_event_sender.send(MessageDebugEvent {
                    sender: peer,
                    event,
                });
            }
            e => {
                tracing::info!("Unhandled event: {e:?}");
            }
        }
        Ok(())
    }
}

pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn subscribe_neighbour_events(
        &self,
    ) -> Result<broadcast::Receiver<MeshTopologyEvent>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::SubscribeNeighbourEvents(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn send_neighbour_event(
        &self,
        peer_id: PeerId,
        event: NeighbourEvent,
    ) -> Result<(), RequestError> {
        self.inner
            .send(Command::SendNeighbourEvent { peer_id, event })
            .await
            .map_err(RequestError::Send)
    }

    pub async fn subscribe_message_events(
        &self,
    ) -> Result<broadcast::Receiver<MessageDebugEvent>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::SubscribeMessageEvents(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn send_message_event(
        &self,
        peer_id: PeerId,
        event: MessageDebugEventType,
    ) -> Result<(), RequestError> {
        self.inner
            .send(Command::SendMessageEvent { peer_id, event })
            .await
            .map_err(RequestError::Send)
    }
}
