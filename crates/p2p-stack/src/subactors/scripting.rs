use std::collections::HashMap;

use docker::Compression;
use libp2p::{
    request_response::{
        cbor, Config, InboundRequestId, Message, OutboundRequestId, ProtocolSupport,
        ResponseChannel,
    },
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use ulid::Ulid;

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    file_transfer::Cid,
    impl_from_special_command,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    DeployImage {
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    DeployedImage { result: Result<Ulid, String> },
}

pub type Behaviour = cbor::Behaviour<Request, Response>;
pub type Event = <Behaviour as NetworkBehaviour>::ToSwarm;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(
            StreamProtocol::new("/scripting/0.1.0"),
            ProtocolSupport::Full,
        )],
        Config::default(),
    )
}

#[derive(Clone, Debug)]
pub enum ActorToClient {
    DeployImage {
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
        id: InboundRequestId,
    },
}

#[derive(Debug)]
pub enum Command {
    DeployImage {
        to: PeerId,
        root_fs: Cid,
        compression: Compression,
        sender: oneshot::Sender<Result<Ulid, String>>,
        ports: Vec<u16>,
    },
    Subscribe(oneshot::Sender<Option<mpsc::Receiver<ActorToClient>>>),
    DeployedImage {
        id: InboundRequestId,
        result: Result<Ulid, String>,
    },
}

impl_from_special_command!(Scripting);

pub struct Actor {
    inflight: HashMap<OutboundRequestId, oneshot::Sender<Result<Ulid, String>>>,
    client_inflight: HashMap<InboundRequestId, ResponseChannel<Response>>,
    to_client_sender: mpsc::Sender<ActorToClient>,
    to_client_receiver: Option<mpsc::Receiver<ActorToClient>>,
}

impl Default for Actor {
    fn default() -> Self {
        let (actor_to_client_sender, actor_to_client_receiver) = mpsc::channel(10);
        Self {
            inflight: HashMap::new(),
            client_inflight: HashMap::new(),
            to_client_sender: actor_to_client_sender,
            to_client_receiver: Some(actor_to_client_receiver),
        }
    }
}

impl SubActor for Actor {
    type SubCommand = Command;
    type Event = <Behaviour as NetworkBehaviour>::ToSwarm;

    type CommandError = void::Void;

    type EventError = void::Void;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Subscribe(sender) => {
                let _ = sender.send(self.to_client_receiver.take());
            }
            Command::DeployImage {
                to,
                root_fs,
                compression,
                sender,
                ports,
            } => {
                let req_id = behaviour.scripting.send_request(
                    &to,
                    Request::DeployImage {
                        root_fs,
                        compression,
                        ports,
                    },
                );
                self.inflight.insert(req_id, sender);
            }
            Command::DeployedImage { id, result } => {
                if let Some(channel) = self.client_inflight.remove(&id) {
                    if let Err(e) = behaviour
                        .scripting
                        .send_response(channel, Response::DeployedImage { result })
                    {
                        tracing::error!(error = ?e, "Failed to send response");
                    }
                }
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
                message:
                    Message::Request {
                        request_id,
                        request:
                            Request::DeployImage {
                                root_fs,
                                compression,
                                ports,
                            },
                        channel,
                    },
                peer,
            } => {
                if let Err(e) = self.to_client_sender.try_send(ActorToClient::DeployImage {
                    root_fs,
                    compression,
                    id: request_id,
                    ports,
                }) {
                    tracing::error!(peer = ?peer, error = ?e, "Failed to send local deployment command");
                } else {
                    self.client_inflight.insert(request_id, channel);
                }
            }
            Event::Message {
                peer,
                message:
                    Message::Response {
                        request_id,
                        response: Response::DeployedImage { result },
                    },
            } => {
                if let Some(sender) = self.inflight.remove(&request_id) {
                    if let Err(e) = sender.send(result) {
                        tracing::error!(error = ?e, "Failed to send result");
                    }
                } else {
                    tracing::error!(peer = ?peer, "Received unexpected response");
                }
            }
            Event::OutboundFailure {
                request_id, error, ..
            } => {
                if let Some(sender) = self.inflight.remove(&request_id) {
                    if let Err(e) = sender.send(Err(error.to_string())) {
                        tracing::error!(error = ?e, "Failed to send error");
                    }
                }
            }
            Event::InboundFailure { .. } | Event::ResponseSent { .. } => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn deploy_image(
        &self,
        to: PeerId,
        root_fs: Cid,
        compression: Compression,
        ports: impl IntoIterator<Item = u16>,
    ) -> Result<Ulid, String> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::DeployImage {
                to,
                root_fs,
                compression,
                sender,
                ports: ports.into_iter().collect(),
            })
            .await
            .expect("Failed to send");
        receiver.await.expect("Failed to receive")
    }

    pub async fn subscribe(&self) -> Result<Option<mpsc::Receiver<ActorToClient>>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe(sender))
            .await
            .expect("Failed to send");
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn deployed_image(
        &self,
        id: InboundRequestId,
        result: Result<Ulid, String>,
    ) -> Result<(), RequestError<Command>> {
        self.inner
            .send(Command::DeployedImage { id, result })
            .await
            .map_err(RequestError::Send)
    }
}
