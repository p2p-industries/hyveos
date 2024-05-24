use std::collections::HashMap;

use libp2p::{
    request_response::{cbor, Config, Event, Message, ProtocolSupport},
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use tokio::sync::mpsc;

use crate::impl_from_special_command;

use super::{actor::SubActor, client::SpecialClient};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ReportRoundTrip {
    nonce: u64,
}

pub type Behaviour = cbor::Behaviour<ReportRoundTrip, ()>;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/round_trip"), ProtocolSupport::Full)],
        Config::default(),
    )
}

pub struct FinishedRoundTrip {
    pub from: PeerId,
}

pub enum Command {
    ReportRoundTrip {
        to: PeerId,
        nonce: u64,
    },
    Register {
        nonce: u64,
        sender: mpsc::Sender<FinishedRoundTrip>,
    },
}

impl_from_special_command!(RoundTrip);

#[derive(Default)]
pub struct Actor {
    senders: HashMap<u64, mpsc::Sender<FinishedRoundTrip>>,
}

impl SubActor for Actor {
    type SubCommand = Command;
    type Event = <Behaviour as NetworkBehaviour>::ToSwarm;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut super::behaviour::MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Register { nonce, sender } => {
                self.senders.insert(nonce, sender);
            }
            Command::ReportRoundTrip { to, nonce } => {
                behaviour
                    .round_trip
                    .send_request(&to, ReportRoundTrip { nonce });
            }
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        _behaviour: &mut super::behaviour::MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::Message {
                peer,
                message: Message::Request { request, .. },
            } => {
                if let Some(sender) = self.senders.get_mut(&request.nonce) {
                    let _ = sender.try_send(FinishedRoundTrip { from: peer });
                } else {
                    tracing::warn!("Received unexpected request: {request:?}");
                }
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
    pub async fn report_round_trip(&self, to: PeerId, nonce: u64) {
        self.inner
            .send(Command::ReportRoundTrip { to, nonce })
            .await
            .expect("Failed to send");
    }

    pub async fn register(&self, nonce: u64) -> mpsc::Receiver<FinishedRoundTrip> {
        let (sender, receiver) = mpsc::channel(10);
        self.inner
            .send(Command::Register { nonce, sender })
            .await
            .expect("Failed to send");
        receiver
    }
}
