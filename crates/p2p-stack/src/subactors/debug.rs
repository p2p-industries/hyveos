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
pub enum NeighbourEvent {
    Init(Vec<PeerId>),
    Discovered(PeerId),
    Lost(PeerId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    NeighbourEvent(NeighbourEvent),
}

pub type Behaviour = cbor::Behaviour<Message, ()>;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/debug"), ProtocolSupport::Full)],
        Config::default(),
    )
}

pub enum Command {
    SubscribeNeighbourEvents(oneshot::Sender<broadcast::Receiver<(PeerId, NeighbourEvent)>>),
    SendNeighbourEvent {
        peer_id: PeerId,
        event: NeighbourEvent,
    },
}

impl_from_special_command!(Debug);

#[derive(Debug)]
pub struct Actor {
    neighbour_event_sender: broadcast::Sender<(PeerId, NeighbourEvent)>,
}

impl Default for Actor {
    fn default() -> Self {
        let (neighbour_event_sender, _) = broadcast::channel(10);
        Self {
            neighbour_event_sender,
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
                let _ = self.neighbour_event_sender.send((peer, event));
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
    ) -> Result<broadcast::Receiver<(PeerId, NeighbourEvent)>, RequestError> {
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
}
