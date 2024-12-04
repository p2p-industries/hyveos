use std::{collections::HashMap, time::Duration};

use docker::Compression;
use hyveos_core::{file_transfer::Cid, scripting::RunningScript};
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
    impl_from_special_command,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    DeployImage {
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
        persistent: bool,
    },
    ListContainers,
    StopContainer {
        id: Ulid,
    },
    StopAllContainers {
        kill: bool,
    },
}

pub type ListContainersResult = Result<Vec<RunningScript>, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    DeployedImage { result: Result<Ulid, String> },
    ListContainers { result: ListContainersResult },
    StopContainer { result: Result<(), String> },
    StopAllContainers { result: Result<(), String> },
}

pub type Behaviour = cbor::Behaviour<Request, Response>;
pub type Event = <Behaviour as NetworkBehaviour>::ToSwarm;

pub fn new() -> Behaviour {
    cbor::Behaviour::new(
        [(
            StreamProtocol::new("/scripting/0.1.0"),
            ProtocolSupport::Full,
        )],
        Config::default().with_request_timeout(REQUEST_TIMEOUT),
    )
}

#[derive(Clone, Debug)]
pub enum ActorToClient {
    DeployImage {
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
        persistent: bool,
        request_id: InboundRequestId,
    },
    ListContainers {
        request_id: InboundRequestId,
    },
    StopContainer {
        request_id: InboundRequestId,
        id: Ulid,
    },
    StopAllContainers {
        request_id: InboundRequestId,
        kill: bool,
    },
}

#[derive(Debug)]
pub enum Command {
    DeployImage {
        to: PeerId,
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
        persistent: bool,
        sender: oneshot::Sender<Result<Ulid, String>>,
    },
    Subscribe(oneshot::Sender<Option<mpsc::Receiver<ActorToClient>>>),
    DeployedImage {
        id: InboundRequestId,
        result: Result<Ulid, String>,
    },
    ListContainers {
        peer_id: PeerId,
        sender: oneshot::Sender<ListContainersResult>,
    },
    ListContainersResponse {
        id: InboundRequestId,
        result: ListContainersResult,
    },
    StopContainer {
        peer_id: PeerId,
        id: Ulid,
        sender: oneshot::Sender<Result<(), String>>,
    },
    StopAllContainers {
        peer_id: PeerId,
        kill: bool,
        sender: oneshot::Sender<Result<(), String>>,
    },
    StopContainerResponse {
        id: InboundRequestId,
        result: Result<(), String>,
    },
}

impl_from_special_command!(Scripting);

pub struct Actor {
    inflight_deploy: HashMap<OutboundRequestId, oneshot::Sender<Result<Ulid, String>>>,
    inflight_list: HashMap<OutboundRequestId, oneshot::Sender<ListContainersResult>>,
    inflight_stop: HashMap<OutboundRequestId, oneshot::Sender<Result<(), String>>>,
    client_inflight: HashMap<InboundRequestId, ResponseChannel<Response>>,
    to_client_sender: mpsc::Sender<ActorToClient>,
    to_client_receiver: Option<mpsc::Receiver<ActorToClient>>,
}

impl Default for Actor {
    fn default() -> Self {
        let (actor_to_client_sender, actor_to_client_receiver) = mpsc::channel(10);
        Self {
            inflight_deploy: HashMap::new(),
            inflight_list: HashMap::new(),
            inflight_stop: HashMap::new(),
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
                ports,
                persistent,
                sender,
            } => {
                let req_id = behaviour.scripting.send_request(
                    &to,
                    Request::DeployImage {
                        root_fs,
                        compression,
                        ports,
                        persistent,
                    },
                );
                self.inflight_deploy.insert(req_id, sender);
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
            Command::ListContainers { peer_id, sender } => {
                let req_id = behaviour
                    .scripting
                    .send_request(&peer_id, Request::ListContainers);
                self.inflight_list.insert(req_id, sender);
            }
            Command::ListContainersResponse { id, result } => {
                if let Some(channel) = self.client_inflight.remove(&id) {
                    if let Err(e) = behaviour
                        .scripting
                        .send_response(channel, Response::ListContainers { result })
                    {
                        tracing::error!(error = ?e, "Failed to send response");
                    }
                }
            }
            Command::StopContainer {
                peer_id,
                id,
                sender,
            } => {
                let req_id = behaviour
                    .scripting
                    .send_request(&peer_id, Request::StopContainer { id });
                self.inflight_stop.insert(req_id, sender);
            }
            Command::StopAllContainers {
                peer_id,
                kill,
                sender,
            } => {
                let req_id = behaviour
                    .scripting
                    .send_request(&peer_id, Request::StopAllContainers { kill });
                self.inflight_stop.insert(req_id, sender);
            }
            Command::StopContainerResponse { id, result } => {
                if let Some(channel) = self.client_inflight.remove(&id) {
                    if let Err(e) = behaviour
                        .scripting
                        .send_response(channel, Response::StopContainer { result })
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
                                persistent,
                            },
                        channel,
                    },
                peer,
            } => {
                if let Err(e) = self.to_client_sender.try_send(ActorToClient::DeployImage {
                    root_fs,
                    compression,
                    ports,
                    persistent,
                    request_id,
                }) {
                    tracing::error!(peer = ?peer, error = ?e, "Failed to send local deployment command");
                } else {
                    self.client_inflight.insert(request_id, channel);
                }
            }
            Event::Message {
                message:
                    Message::Request {
                        request_id,
                        request: Request::ListContainers,
                        channel,
                    },
                peer,
            } => {
                if let Err(e) = self
                    .to_client_sender
                    .try_send(ActorToClient::ListContainers { request_id })
                {
                    tracing::error!(peer = ?peer, error = ?e, "Failed to send list containers command");
                } else {
                    self.client_inflight.insert(request_id, channel);
                }
            }
            Event::Message {
                message:
                    Message::Request {
                        request_id,
                        request: Request::StopContainer { id },
                        channel,
                    },
                peer,
            } => {
                if let Err(e) = self
                    .to_client_sender
                    .try_send(ActorToClient::StopContainer { request_id, id })
                {
                    tracing::error!(peer = ?peer, error = ?e, "Failed to send stop container command");
                } else {
                    self.client_inflight.insert(request_id, channel);
                }
            }
            Event::Message {
                message:
                    Message::Request {
                        request_id,
                        request: Request::StopAllContainers { kill },
                        channel,
                    },
                peer,
            } => {
                if let Err(e) = self
                    .to_client_sender
                    .try_send(ActorToClient::StopAllContainers { request_id, kill })
                {
                    tracing::error!(peer = ?peer, error = ?e, "Failed to send stop all containers command");
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
                if let Some(sender) = self.inflight_deploy.remove(&request_id) {
                    if let Err(e) = sender.send(result) {
                        tracing::error!(error = ?e, "Failed to send result");
                    }
                } else {
                    tracing::error!(peer = ?peer, "Received unexpected response");
                }
            }
            Event::Message {
                peer,
                message:
                    Message::Response {
                        request_id,
                        response: Response::ListContainers { result },
                    },
            } => {
                if let Some(sender) = self.inflight_list.remove(&request_id) {
                    if let Err(e) = sender.send(result) {
                        tracing::error!(error = ?e, "Failed to send result");
                    }
                } else {
                    tracing::error!(peer = ?peer, "Received unexpected response");
                }
            }
            Event::Message {
                peer,
                message:
                    Message::Response {
                        request_id,
                        response:
                            Response::StopContainer { result } | Response::StopAllContainers { result },
                    },
            } => {
                if let Some(sender) = self.inflight_stop.remove(&request_id) {
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
                if let Some(sender) = self.inflight_deploy.remove(&request_id) {
                    if let Err(e) = sender.send(Err(error.to_string())) {
                        tracing::error!(error = ?e, "Failed to send error");
                    }
                } else if let Some(sender) = self.inflight_list.remove(&request_id) {
                    if let Err(e) = sender.send(Err(error.to_string())) {
                        tracing::error!(error = ?e, "Failed to send error");
                    }
                } else if let Some(sender) = self.inflight_stop.remove(&request_id) {
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
        persistent: bool,
    ) -> Result<Ulid, String> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::DeployImage {
                to,
                root_fs,
                compression,
                ports: ports.into_iter().collect(),
                persistent,
                sender,
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

    pub async fn list_containers(&self, peer_id: PeerId) -> ListContainersResult {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::ListContainers { peer_id, sender })
            .await
            .expect("Failed to send");
        receiver.await.expect("Failed to receive")
    }

    pub async fn send_list_containers_response(
        &self,
        id: InboundRequestId,
        result: ListContainersResult,
    ) -> Result<(), RequestError<Command>> {
        self.inner
            .send(Command::ListContainersResponse { id, result })
            .await
            .map_err(RequestError::Send)
    }

    pub async fn stop_container(&self, peer_id: PeerId, id: Ulid) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::StopContainer {
                peer_id,
                id,
                sender,
            })
            .await
            .expect("Failed to send");
        receiver.await.expect("Failed to receive")
    }

    pub async fn stop_all_containers(&self, peer_id: PeerId, kill: bool) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::StopAllContainers {
                peer_id,
                kill,
                sender,
            })
            .await
            .expect("Failed to send");
        receiver.await.expect("Failed to receive")
    }

    pub async fn send_stop_container_response(
        &self,
        id: InboundRequestId,
        result: Result<(), String>,
    ) -> Result<(), RequestError<Command>> {
        self.inner
            .send(Command::StopContainerResponse { id, result })
            .await
            .map_err(RequestError::Send)
    }
}
