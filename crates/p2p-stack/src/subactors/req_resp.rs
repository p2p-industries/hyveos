use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};

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
use tokio::sync::{mpsc, oneshot};

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    impl_from_special_command,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

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
pub enum ResponseError {
    Timeout,
    TopicNotSubscribed(Option<Arc<str>>),
    Script(String),
}

impl From<String> for ResponseError {
    fn from(e: String) -> Self {
        ResponseError::Script(e)
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::Timeout => write!(f, "Request timed out"),
            ResponseError::TopicNotSubscribed(Some(topic)) => {
                write!(f, "Peer is not subscribed to topic '{topic}'")
            }
            ResponseError::TopicNotSubscribed(None) => {
                write!(f, "Peer is not subscribed to the empty topic")
            }
            ResponseError::Script(e) => write!(f, "Script error: {e}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Data(Vec<u8>),
    Error(ResponseError),
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
        Config::default().with_request_timeout(REQUEST_TIMEOUT),
    )
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SubscriptionId(u64);

pub enum Command {
    Request {
        peer_id: PeerId,
        req: Request,
        sender: oneshot::Sender<Response>,
    },
    Subscribe {
        query: Option<TopicQuery>,
        sender: oneshot::Sender<(SubscriptionId, mpsc::Receiver<InboundRequest>)>,
    },
    Unsubscribe(SubscriptionId),
    Respond {
        id: u64,
        response: Response,
    },
}

impl_from_special_command!(ReqResp);

#[derive(Debug, Default)]
pub struct Actor {
    peer_id: Option<PeerId>,
    response_senders: HashMap<OutboundRequestId, oneshot::Sender<Response>>,
    request_subscriptions: HashMap<u64, (Option<TopicQuery>, mpsc::Sender<InboundRequest>)>,
    response_channels: HashMap<u64, Result<ResponseChannel<Response>, oneshot::Sender<Response>>>,
    next_subscription_id: u64,
}

impl SubActor for Actor {
    type SubCommand = Command;
    type Event = <Behaviour as NetworkBehaviour>::ToSwarm;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id: Some(peer_id),
            ..Default::default()
        }
    }

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Request {
                peer_id,
                req,
                sender,
            } => {
                if let Some(own_id) = self.peer_id {
                    if own_id == peer_id {
                        tracing::debug!("Sending self request");
                        let id = rand::random();
                        let topic = req.topic.clone();

                        let request = InboundRequest { id, peer_id, req };

                        let mut sent_to_subscriber = false;
                        for (query, sender) in self.request_subscriptions.values() {
                            let is_match = match (query.as_ref(), topic.as_ref()) {
                                (Some(query), Some(topic)) => query.is_match(topic),
                                (None, None) => true,
                                _ => false,
                            };

                            if is_match && sender.try_send(request.clone()).is_ok() {
                                sent_to_subscriber = true;
                            }
                        }

                        if sent_to_subscriber {
                            self.response_channels.insert(id, Err(sender));
                        } else {
                            let response =
                                Response::Error(ResponseError::TopicNotSubscribed(topic));
                            let _ = sender.send(response);
                        }

                        return Ok(());
                    }
                }

                tracing::debug!("Sending request to peer {peer_id}");
                let id = behaviour.req_resp.send_request(&peer_id, req);
                self.response_senders.insert(id, sender);
            }
            Command::Subscribe { query, sender } => {
                let id = self.next_subscription_id;
                self.next_subscription_id += 1;

                let (request_sender, request_receiver) = mpsc::channel(10);

                self.request_subscriptions
                    .insert(id, (query, request_sender));

                let _ = sender.send((SubscriptionId(id), request_receiver));
            }
            Command::Unsubscribe(id) => {
                self.request_subscriptions.remove(&id.0);
            }
            Command::Respond { id, response } => match self.response_channels.remove(&id) {
                Some(Ok(channel)) => {
                    tracing::debug!("Responding to request with id {id}");
                    let _ = behaviour.req_resp.send_response(channel, response);
                }
                Some(Err(sender)) => {
                    tracing::debug!("Responding to self request with id {id}");
                    let _ = sender.send(response);
                }
                None => {
                    tracing::warn!("Response with id {id} not found");
                }
            },
        }

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
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
                    let topic = req.topic.clone();

                    tracing::debug!("Received request with id {id} from peer {peer}");

                    let request = InboundRequest {
                        id,
                        peer_id: peer,
                        req,
                    };

                    let mut sent_to_subscriber = false;
                    for (query, sender) in self.request_subscriptions.values() {
                        let is_match = match (query.as_ref(), topic.as_ref()) {
                            (Some(query), Some(topic)) => query.is_match(topic),
                            (None, None) => true,
                            _ => false,
                        };

                        if is_match && sender.try_send(request.clone()).is_ok() {
                            sent_to_subscriber = true;
                        }
                    }

                    if sent_to_subscriber {
                        self.response_channels.insert(id, Ok(channel));
                    } else {
                        let response = Response::Error(ResponseError::TopicNotSubscribed(topic));
                        let _ = behaviour.req_resp.send_response(channel, response);
                    }
                }
                Message::Response {
                    request_id,
                    response,
                } => {
                    tracing::debug!("Received response for request with id {request_id}");

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

        tokio::time::timeout(REQUEST_TIMEOUT, receiver)
            .await
            .unwrap_or(Ok(Response::Error(ResponseError::Timeout)))
            .map_err(RequestError::Oneshot)
    }

    pub async fn subscribe(
        &self,
        query: Option<TopicQuery>,
    ) -> Result<(SubscriptionId, mpsc::Receiver<InboundRequest>), RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe { query, sender })
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), RequestError> {
        self.inner
            .send(Command::Unsubscribe(id))
            .await
            .map_err(RequestError::Send)
    }

    pub async fn send_response(&self, id: u64, response: Response) -> Result<(), RequestError> {
        self.inner
            .send(Command::Respond { id, response })
            .await
            .map_err(RequestError::Send)
    }
}
