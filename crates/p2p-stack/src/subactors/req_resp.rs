use std::{collections::HashMap, time::Duration};

use hyveos_core::{
    debug::{MessageDebugEventType, RequestDebugEvent, ResponseDebugEvent},
    req_resp::{self, InboundRequest, Response, ResponseError, TopicQuery},
};
use libp2p::{
    request_response::{
        cbor, Config, Event, InboundRequestId, Message, OutboundRequestId, ProtocolSupport,
        ResponseChannel,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};
use ulid::Ulid;

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    impl_from_special_command,
};

// TODO: lower timeout (requires changes to file_transfer actor)
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    debug_id: Ulid,
    req: req_resp::Request,
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
        req: req_resp::Request,
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
    DebugSubscribe(oneshot::Sender<broadcast::Receiver<MessageDebugEventType>>),
}

impl_from_special_command!(ReqResp);

type ForeignOrSelfResponseChannel =
    Result<(Ulid, ResponseChannel<Response>), oneshot::Sender<Response>>;

#[derive(Debug, Default)]
pub struct Actor {
    peer_id: Option<PeerId>,
    response_senders: HashMap<OutboundRequestId, oneshot::Sender<Response>>,
    request_subscriptions: HashMap<u64, (Option<TopicQuery>, mpsc::Sender<InboundRequest>)>,
    response_channels: HashMap<u64, ForeignOrSelfResponseChannel>,
    next_subscription_id: u64,
    debug_sender: Option<broadcast::Sender<MessageDebugEventType>>,
}

impl Actor {
    fn send_debug_event(&mut self, f: impl FnOnce() -> MessageDebugEventType) {
        if let Some(debug_sender) = self.debug_sender.take() {
            if debug_sender.send(f()).is_ok() {
                // If there are still subscribers, put the sender back
                self.debug_sender = Some(debug_sender);
            }
        }
    }
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
                                (Some(query), Some(topic)) => query.matches(topic),
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

                let debug_id = Ulid::new();

                self.send_debug_event(|| {
                    MessageDebugEventType::Request(RequestDebugEvent {
                        id: debug_id,
                        receiver: peer_id,
                        msg: req.clone(),
                    })
                });

                let req = Request { debug_id, req };

                let outbound_id = behaviour.req_resp.send_request(&peer_id, req);
                self.response_senders.insert(outbound_id, sender);
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
                Some(Ok((debug_id, channel))) => {
                    tracing::debug!("Responding to request with id {id}");

                    self.send_debug_event(|| {
                        MessageDebugEventType::Response(ResponseDebugEvent {
                            req_id: debug_id,
                            response: response.clone(),
                        })
                    });

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
            Command::DebugSubscribe(sender) => {
                let receiver = self
                    .debug_sender
                    .get_or_insert_with(|| broadcast::channel(5).0)
                    .subscribe();
                let _ = sender.send(receiver);
            }
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
                    request: Request { debug_id, req },
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
                            (Some(query), Some(topic)) => query.matches(topic),
                            (None, None) => true,
                            _ => false,
                        };

                        if is_match && sender.try_send(request.clone()).is_ok() {
                            sent_to_subscriber = true;
                        }
                    }

                    if sent_to_subscriber {
                        self.response_channels.insert(id, Ok((debug_id, channel)));
                    } else {
                        let response = Response::Error(ResponseError::TopicNotSubscribed(topic));

                        self.send_debug_event(|| {
                            MessageDebugEventType::Response(ResponseDebugEvent {
                                req_id: debug_id,
                                response: response.clone(),
                            })
                        });

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
        req: req_resp::Request,
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

    pub async fn debug_subscribe(
        &self,
    ) -> Result<broadcast::Receiver<MessageDebugEventType>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::DebugSubscribe(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }
}
