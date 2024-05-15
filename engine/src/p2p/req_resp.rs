use std::collections::HashMap;

use libp2p::{
    request_response::{
        cbor, Config, Event, Message, OutboundRequestId, ProtocolSupport, ResponseChannel,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use tokio::sync::{broadcast, oneshot};

use super::{
    actor::SubActor,
    client::{RequestError, SpecialClient},
};
use crate::impl_from_special_command;

#[derive(Debug, Clone)]
pub struct InboundRequest {
    pub id: u64,
    pub peer_id: PeerId,
    pub data: Vec<u8>,
}

pub type Behaviour = cbor::Behaviour<Vec<u8>, Vec<u8>>;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/req_resp"), ProtocolSupport::Full)],
        Config::default(),
    )
}

pub enum Command {
    Request {
        peer_id: PeerId,
        data: Vec<u8>,
        sender: oneshot::Sender<Vec<u8>>,
    },
    Subscribe(oneshot::Sender<broadcast::Receiver<InboundRequest>>),
    Respond {
        id: u64,
        data: Vec<u8>,
    },
}

impl_from_special_command!(ReqResp);

#[derive(Debug)]
pub struct Actor {
    response_senders: HashMap<OutboundRequestId, oneshot::Sender<Vec<u8>>>,
    request_sender: broadcast::Sender<InboundRequest>,
    response_channels: HashMap<u64, ResponseChannel<Vec<u8>>>,
}

impl Default for Actor {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(10);
        Self {
            response_senders: HashMap::new(),
            request_sender: sender,
            response_channels: HashMap::new(),
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
        behaviour: &mut super::behaviour::MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Request {
                peer_id,
                data,
                sender,
            } => {
                let id = behaviour.req_resp.send_request(&peer_id, data);
                self.response_senders.insert(id, sender);
            }
            Command::Subscribe(sender) => {
                let _ = sender.send(self.request_sender.subscribe());
            }
            Command::Respond { id, data } => {
                if let Some(channel) = self.response_channels.remove(&id) {
                    behaviour.req_resp.send_response(channel, data);
                }
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
            Event::Message { peer, message } => match message {
                Message::Request {
                    request_id,
                    request: data,
                    channel,
                } => {
                    let id = unsafe { std::mem::transmute(request_id) };

                    let request = InboundRequest {
                        id,
                        peer_id: peer,
                        data,
                    };

                    self.response_channels.insert(id, channel);
                    self.request_sender.send(request).unwrap();
                }
                Message::Response {
                    request_id,
                    response,
                } => {
                    if let Some(sender) = self.response_senders.remove(&request_id) {
                        sender.send(response).unwrap();
                    }
                }
            },
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
    pub async fn send_request(
        &self,
        peer_id: PeerId,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Request {
                peer_id,
                data,
                sender,
            })
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn subscribe(&self) -> Result<broadcast::Receiver<InboundRequest>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn send_response(&self, id: u64, data: Vec<u8>) -> Result<(), RequestError> {
        self.inner
            .send(Command::Respond { id, data })
            .await
            .map_err(RequestError::Send)
    }
}
