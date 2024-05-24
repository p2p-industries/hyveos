use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::{future, Stream, TryStreamExt as _};
use libp2p::{
    request_response::{
        cbor, Config, Event, InboundRequestId, Message, OutboundRequestId, ProtocolSupport,
        ResponseChannel,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, oneshot};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use super::{
    actor::SubActor,
    client::{RequestError, SpecialClient},
};
use crate::impl_from_special_command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub data: Vec<u8>,
    pub topic: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct InboundRequest {
    pub id: u64,
    pub peer_id: PeerId,
    pub req: Request,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Data(Vec<u8>),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum TopicQuery {
    Regex(Regex),
    String(Arc<str>),
}

impl TopicQuery {
    pub fn is_match(&self, topic: impl AsRef<str>) -> bool {
        match self {
            TopicQuery::Regex(regex) => regex.is_match(topic.as_ref()),
            TopicQuery::String(query) => query.as_ref() == topic.as_ref(),
        }
    }
}

pub type Behaviour = cbor::Behaviour<Request, Response>;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(StreamProtocol::new("/req_resp"), ProtocolSupport::Full)],
        Config::default(),
    )
}

pub type InboundRequestStream =
    Pin<Box<dyn Stream<Item = Result<InboundRequest, BroadcastStreamRecvError>> + Send>>;

pub enum Command {
    Request {
        peer_id: PeerId,
        req: Request,
        sender: oneshot::Sender<Response>,
    },
    Subscribe {
        query: Option<TopicQuery>,
        sender: oneshot::Sender<InboundRequestStream>,
    },
    Respond {
        id: u64,
        response: Response,
    },
}

impl_from_special_command!(ReqResp);

#[derive(Debug)]
pub struct Actor {
    response_senders: HashMap<OutboundRequestId, oneshot::Sender<Response>>,
    request_sender: broadcast::Sender<InboundRequest>,
    response_channels: HashMap<u64, ResponseChannel<Response>>,
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
                req,
                sender,
            } => {
                let id = behaviour.req_resp.send_request(&peer_id, req);
                self.response_senders.insert(id, sender);
            }
            Command::Subscribe { query, sender } => {
                let receiver = self.request_sender.subscribe();

                let stream = BroadcastStream::new(receiver).try_filter(move |req| {
                    future::ready(match (query.as_ref(), req.req.topic.as_ref()) {
                        (Some(query), Some(topic)) => query.is_match(topic),
                        (None, None) => true,
                        _ => false,
                    })
                });

                let _ = sender.send(Box::pin(stream));
            }
            Command::Respond { id, response } => {
                if let Some(channel) = self.response_channels.remove(&id) {
                    _ = behaviour.req_resp.send_response(channel, response);
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
                    request: req,
                    channel,
                } => {
                    // This is safe because InboundRequestId is a newtype around u64
                    let id = unsafe { std::mem::transmute::<InboundRequestId, u64>(request_id) };

                    let request = InboundRequest {
                        id,
                        peer_id: peer,
                        req,
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
                tracing::debug!("Unhandled event: {e:?}");
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
        req: Request,
    ) -> Result<Response, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Request {
                peer_id,
                req,
                sender,
            })
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn subscribe(
        &self,
        query: Option<TopicQuery>,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<InboundRequest, BroadcastStreamRecvError>> + Send>>,
        RequestError,
    > {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe { query, sender })
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn send_response(&self, id: u64, response: Response) -> Result<(), RequestError> {
        self.inner
            .send(Command::Respond { id, response })
            .await
            .map_err(RequestError::Send)
    }
}
