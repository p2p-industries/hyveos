use std::{collections::HashMap, time::Duration};

use libp2p::{
    request_response::{
        cbor, Config, Message, OutboundFailure, OutboundRequestId, ProtocolSupport,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use tokio::{sync::oneshot, time::Instant};

use crate::{
    actor::SubActor, behaviour::MyBehaviour, client::SpecialClient, impl_from_special_command,
};

#[derive(Debug)]
struct PingTracker {
    start: Instant,
    resp_channel: oneshot::Sender<Result<Duration, OutboundFailure>>,
}

#[derive(Debug, Default)]
pub struct Actor {
    tracker: HashMap<OutboundRequestId, PingTracker>,
}

#[derive(Debug)]
pub enum Command {
    Ping {
        peer: PeerId,
        resp_channel: oneshot::Sender<Result<Duration, OutboundFailure>>,
        start: Instant,
    },
}

pub type Behaviour = cbor::Behaviour<(), ()>;
pub type Event = <Behaviour as NetworkBehaviour>::ToSwarm;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/fast_ping"), ProtocolSupport::Full)],
        Config::default(),
    )
}

impl_from_special_command!(Ping);

impl SubActor for Actor {
    type Event = Event;
    type SubCommand = Command;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::Message { message, .. } => match message {
                Message::Request {
                    request, channel, ..
                } => {
                    let _ = behaviour.ping.send_response(channel, request);
                }
                Message::Response { request_id, .. } => {
                    if let Some(PingTracker {
                        start,
                        resp_channel,
                    }) = self.tracker.remove(&request_id)
                    {
                        let duration = start.elapsed();
                        let _ = resp_channel.send(Ok(duration));
                    }
                }
            },
            Event::OutboundFailure {
                request_id, error, ..
            } => {
                if let Some(PingTracker { resp_channel, .. }) = self.tracker.remove(&request_id) {
                    let _ = resp_channel.send(Err(error));
                }
            }
            Event::InboundFailure { .. } | Event::ResponseSent { .. } => {}
        }
        Ok(())
    }

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Ping {
                peer,
                resp_channel,
                start,
            } => {
                let request_id = behaviour.ping.send_request(&peer, ());
                self.tracker.insert(
                    request_id,
                    PingTracker {
                        start,
                        resp_channel,
                    },
                );
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
    pub async fn ping(&self, peer: PeerId) -> Result<Duration, OutboundFailure> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Ping {
                peer,
                resp_channel: sender,
                start: Instant::now(),
            })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
}
