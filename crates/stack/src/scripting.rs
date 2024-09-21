use std::{
    collections::HashMap,
    env::temp_dir,
    future::Future,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[cfg(feature = "batman")]
use bridge::DebugCommandSender;
use bridge::{Bridge, Error as BridgeError, CONTAINER_SHARED_DIR};
use bytes::Bytes;
use clap::ValueEnum;
use docker::{Compression, ContainerManager, NetworkMode, PulledImage, StoppedContainer};
use futures::{
    future::{try_maybe_done, OptionFuture, TryMaybeDone},
    stream::{FuturesUnordered, StreamExt as _, TryStreamExt as _},
    FutureExt as _,
};
use libp2p::PeerId;
use p2p_industries_core::{file_transfer::Cid, scripting::RunningScript};
use p2p_stack::{file_transfer, scripting::ActorToClient, Client as P2PClient};
use serde::Deserialize;
use tokio::{
    fs::{metadata, File},
    io::{stderr, stdout, AsyncReadExt as _, AsyncWriteExt as _, BufReader},
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use ulid::Ulid;

use crate::{
    db::{self, Client as DbClient},
    future_map::FutureMap,
    printer::SharedPrinter,
};

const CONTAINER_BRIDGE_SOCKET: &str = "/var/run/bridge.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
pub enum ScriptManagementConfig {
    Allow,
    Deny,
}

impl Default for ScriptManagementConfig {
    fn default() -> Self {
        Self::Deny
    }
}

enum SelfCommand {
    DeployImage {
        image: PulledImage<'static>,
        ports: Vec<u16>,
        persistent: bool,
        sender: oneshot::Sender<Result<Ulid, ExecutionError>>,
    },
    ListContainers {
        sender: oneshot::Sender<Vec<RunningScript>>,
    },
    StopContainer {
        container_id: Ulid,
        sender: oneshot::Sender<Result<(), ExecutionError>>,
    },
    StopAllContainers {
        kill: bool,
        sender: oneshot::Sender<Result<(), ExecutionError>>,
    },
}

pub struct ScriptingManagerBuilder {
    command_broker: mpsc::Receiver<ActorToClient>,
    client: P2PClient,
    db_client: DbClient,
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
    script_management: ScriptManagementConfig,
    shared_printer: Option<SharedPrinter>,
}

impl ScriptingManagerBuilder {
    pub fn new(
        command_broker: mpsc::Receiver<ActorToClient>,
        client: P2PClient,
        db_client: DbClient,
        base_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        script_management: ScriptManagementConfig,
    ) -> Self {
        Self {
            command_broker,
            client,
            db_client,
            base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
            script_management,
            shared_printer: None,
        }
    }

    pub fn with_printer(mut self, printer: SharedPrinter) -> Self {
        self.shared_printer = Some(printer);
        self
    }

    pub fn build(self) -> (ScriptingManager, ScriptingClient) {
        let Self {
            command_broker,
            client,
            db_client,
            base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
            script_management,
            shared_printer,
        } = self;
        let (self_command_sender, self_command_receiver) = mpsc::channel(1);

        let scripting_client = ScriptingClient::new(client.clone(), self_command_sender);

        let manager = {
            let scripting_client = if let ScriptManagementConfig::Allow = script_management {
                Some(scripting_client.clone())
            } else {
                None
            };

            ScriptingManager {
                command_broker,
                self_command_receiver,
                container_manager: ContainerManager::new()
                    .expect("Failed to create container manager"),
                client,
                db_client,
                base_path,
                #[cfg(feature = "batman")]
                debug_command_sender,
                scripting_client,
                shared_printer,
                container_handles: FutureMap::new(),
            }
        };

        (manager, scripting_client)
    }
}

pub struct ScriptingManager {
    command_broker: mpsc::Receiver<ActorToClient>,
    self_command_receiver: mpsc::Receiver<SelfCommand>,
    container_manager: ContainerManager,
    client: P2PClient,
    db_client: DbClient,
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
    scripting_client: Option<ScriptingClient>,
    shared_printer: Option<SharedPrinter>,
    container_handles: FutureMap<Ulid, ContainerHandle>,
}

impl ScriptingManager {
    fn execution_manager(&self) -> ExecutionManager {
        ExecutionManager {
            container_manger: &self.container_manager,
            client: self.client.clone(),
            db_client: self.db_client.clone(),
            base_path: self.base_path.clone(),
            #[cfg(feature = "batman")]
            debug_command_sender: self.debug_command_sender.clone(),
            scripting_client: self.scripting_client.clone(),
            shared_printer: self.shared_printer.clone(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(command) = self.command_broker.recv() => {
                    self.handle_command(command).await;
                },
                Some(command) = self.self_command_receiver.recv() => {
                    self.handle_self_command(command).await;
                },
                Some((_, res)) = self.container_handles.next() => {
                    match res {
                        Ok((id, container)) => {
                            tracing::info!(
                                "Container exited: {id} ({})",
                                container.image.image
                            );
                        }
                        Err(e) => {
                            tracing::error!("Container exited with error: {e}");
                        }
                    }
                }
                else => { break }
            }
        }
    }

    async fn handle_command(&mut self, command: ActorToClient) {
        match command {
            ActorToClient::DeployImage {
                root_fs,
                compression,
                ports,
                persistent,
                request_id,
            } => {
                let persisted_ports = persistent.then(|| ports.clone());

                let handle = self
                    .execution_manager()
                    .exec_foreign(root_fs, compression, ports)
                    .await;

                let scripting = self.client.scripting().clone();

                self.add_handle(handle, persisted_ports, move |id| async move {
                    scripting
                        .deployed_image(request_id, id.map_err(|e| e.to_string()))
                        .map(|_| ())
                        .await;
                })
                .await;
            }
            ActorToClient::ListContainers { request_id } => {
                let result = Ok(self.list_containers());
                let _ = self
                    .client
                    .scripting()
                    .send_list_containers_response(request_id, result)
                    .await;
            }
            ActorToClient::StopContainer { request_id, id } => {
                let result = self.stop_container(id).await.map_err(|e| e.to_string());
                let _ = self
                    .client
                    .scripting()
                    .send_stop_container_response(request_id, result)
                    .await;
            }
            ActorToClient::StopAllContainers { request_id, kill } => {
                let result = self
                    .stop_all_containers(kill)
                    .await
                    .map_err(|e| e.to_string());
                let _ = self
                    .client
                    .scripting()
                    .send_stop_container_response(request_id, result)
                    .await;
            }
        }
    }

    async fn handle_self_command(&mut self, command: SelfCommand) {
        match command {
            SelfCommand::DeployImage {
                image,
                ports,
                persistent,
                sender,
            } => {
                let persisted_ports = persistent.then(|| ports.clone());

                let handle = self.execution_manager().exec(image, ports).await;

                self.add_handle(handle, persisted_ports, move |id| async move {
                    let _ = sender.send(id);
                })
                .await;
            }
            SelfCommand::ListContainers { sender } => {
                let _ = sender.send(self.list_containers());
            }
            SelfCommand::StopContainer {
                container_id,
                sender,
            } => {
                let _ = sender.send(self.stop_container(container_id).await);
            }
            SelfCommand::StopAllContainers { kill, sender } => {
                let _ = sender.send(self.stop_all_containers(kill).await);
            }
        }
    }

    async fn add_handle<Fut: Future<Output = ()>>(
        &mut self,
        handle: Result<ContainerHandle, ExecutionError>,
        persisted_ports: Option<Vec<u16>>,
        send: impl FnOnce(Result<Ulid, ExecutionError>) -> Fut,
    ) {
        let id = match handle {
            Ok(handle) => {
                let id = handle.id;
                let image_name = handle.image_name.clone();
                self.container_handles.insert(id, handle);

                if let Some(ports) = persisted_ports {
                    if let Err(e) = self
                        .db_client
                        .insert_startup_script(image_name.as_ref(), ports)
                    {
                        Err(e.into())
                    } else {
                        Ok(id)
                    }
                } else {
                    Ok(id)
                }
            }
            Err(e) => {
                tracing::error!("Failed to deploy image: {e}");
                Err(e)
            }
        };

        send(id).await;
    }

    fn list_containers(&self) -> Vec<RunningScript> {
        self.container_handles
            .iter()
            .map(|(id, handle)| RunningScript {
                id: *id,
                image: handle.image_name.clone(),
                name: handle.script_name.clone(),
            })
            .collect()
    }

    async fn stop_container(&mut self, container_id: Ulid) -> Result<(), ExecutionError> {
        if let Some(handle) = self.container_handles.remove(&container_id) {
            match handle.stop(false).await {
                Ok((_, container)) => {
                    tracing::info!(
                        "Stopped container: {container_id} ({})",
                        container.image.image
                    );
                    self.db_client
                        .remove_startup_script(container.image.image)?;
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to stop container: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(ExecutionError::ContainerNotFound(container_id))
        }
    }

    async fn stop_all_containers(&mut self, kill: bool) -> Result<(), ExecutionError> {
        let containers = self
            .container_handles
            .take_futures()
            .map(|handle| handle.stop(kill))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await;

        match containers {
            Ok(containers) => {
                tracing::info!(
                    "Stopped all containers: {}",
                    containers
                        .iter()
                        .map(|(id, container)| format!("{id} ({})", container.image.image))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("File transfer error: `{0}`")]
    FileTransfer(#[from] file_transfer::ClientError),
    #[error("Io error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Docker error: `{0}`")]
    Docker(#[from] docker::Error),
    #[error("Bridge error: `{0}`")]
    Bridge(#[from] BridgeError),
    #[error("Shared directory path is not a valid utf-8 string")]
    SharedDirPathInvalid(PathBuf),
    #[error("Container not found: `{0}`")]
    ContainerNotFound(Ulid),
    #[error("Join error: `{0}`")]
    Join(#[from] tokio::task::JoinError),
    #[error("Remote deploy error: `{0}`")]
    RemoteDeployError(String),
    #[error("Self deploy error: `{0}`")]
    SelfDeployError(String),
    #[error("Container list error: `{0}`")]
    ListContainersError(String),
    #[error("Container stop error: `{0}`")]
    StopContainerError(String),
    #[error("Error persisting script: `{0}`")]
    Persistence(#[from] db::Error),
}

#[pin_project::pin_project]
struct ContainerHandle {
    id: Ulid,
    image_name: Arc<str>,
    script_name: Option<Arc<str>>,
    stop_sender: oneshot::Sender<bool>,
    #[pin]
    handle: TryMaybeDone<JoinHandle<Result<StoppedContainer<'static>, ExecutionError>>>,
    #[pin]
    bridge_handle: TryMaybeDone<JoinHandle<Result<(), BridgeError>>>,
}

impl ContainerHandle {
    fn new(
        id: Ulid,
        image_name: Arc<str>,
        script_name: Option<Arc<str>>,
        stop_sender: oneshot::Sender<bool>,
        handle: JoinHandle<Result<StoppedContainer<'static>, ExecutionError>>,
        bridge_handle: JoinHandle<Result<(), BridgeError>>,
    ) -> Self {
        Self {
            id,
            image_name,
            script_name,
            stop_sender,
            handle: try_maybe_done(handle),
            bridge_handle: try_maybe_done(bridge_handle),
        }
    }

    async fn stop(
        mut self,
        kill: bool,
    ) -> Result<(Ulid, StoppedContainer<'static>), ExecutionError> {
        let _ = self.stop_sender.send(kill);
        tokio::try_join!(&mut self.handle, &mut self.bridge_handle)?;
        match (
            Pin::new(&mut self.handle).take_output().unwrap(),
            Pin::new(&mut self.bridge_handle).take_output().unwrap(),
        ) {
            (Ok(container), Ok(())) => Ok((self.id, container)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e.into()),
        }
    }
}

impl Future for ContainerHandle {
    type Output = Result<(Ulid, StoppedContainer<'static>), ExecutionError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let id = self.id;
        let mut this = self.project();
        if this.handle.as_mut().poll(cx)?.is_ready()
            && this.bridge_handle.as_mut().poll(cx)?.is_ready()
        {
            Poll::Ready(
                match (
                    this.handle.take_output().unwrap(),
                    this.bridge_handle.take_output().unwrap(),
                ) {
                    (Ok(container), Ok(())) => Ok((id, container)),
                    (Err(e), _) => Err(e),
                    (_, Err(e)) => Err(e.into()),
                },
            )
        } else {
            Poll::Pending
        }
    }
}

struct ExecutionManager<'a> {
    container_manger: &'a ContainerManager,
    client: P2PClient,
    db_client: DbClient,
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
    scripting_client: Option<ScriptingClient>,
    shared_printer: Option<SharedPrinter>,
}

impl<'a> ExecutionManager<'a> {
    async fn fetch_root_fs(&self, cid: Cid) -> Result<Bytes, ExecutionError> {
        let path = self.client.file_transfer().get_cid(cid).await?;
        let mut file = BufReader::new(File::open(&path).await?);
        let mut buf = Vec::with_capacity(
            metadata(&path)
                .await?
                .len()
                .try_into()
                .expect("File is too large to fit into memory."),
        );
        file.read_to_end(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn exec_foreign(
        self,
        root_fs: Cid,
        compression: Compression,
        ports: Vec<u16>,
    ) -> Result<ContainerHandle, ExecutionError> {
        let root_fs = self.fetch_root_fs(root_fs).await?;
        let pulled_image = self
            .container_manger
            .import_image(root_fs, compression)
            .await?;

        self.exec(pulled_image, ports).await
    }

    async fn exec(
        self,
        image: PulledImage<'_>,
        ports: Vec<u16>,
    ) -> Result<ContainerHandle, ExecutionError> {
        if let Some(scripting_client) = self.scripting_client {
            let bridge = Bridge::new(
                self.client,
                self.db_client,
                self.base_path,
                #[cfg(feature = "batman")]
                self.debug_command_sender,
                scripting_client,
            )
            .await?;

            Self::exec_with_bridge(bridge, image, ports, self.shared_printer).await
        } else {
            let bridge = Bridge::new(
                self.client,
                self.db_client,
                self.base_path,
                #[cfg(feature = "batman")]
                self.debug_command_sender,
                ForbiddenScriptingClient,
            )
            .await?;

            Self::exec_with_bridge(bridge, image, ports, self.shared_printer).await
        }
    }

    async fn exec_with_bridge(
        bridge: Bridge<DbClient, impl bridge::ScriptingClient>,
        image: PulledImage<'_>,
        ports: Vec<u16>,
        shared_printer: Option<SharedPrinter>,
    ) -> Result<ContainerHandle, ExecutionError> {
        let Bridge {
            client,
            cancellation_token,
            ulid,
            socket_path,
            shared_dir_path,
        } = bridge;

        let image_name = Arc::from(&*image.image);

        let script_name = image
            .get_label("industries.p2p.script.name")
            .await?
            .map(Into::into);

        let volumes = [
            (
                shared_dir_path
                    .clone()
                    .as_path()
                    .to_str()
                    .ok_or(ExecutionError::SharedDirPathInvalid(shared_dir_path))?
                    .to_string(),
                CONTAINER_SHARED_DIR.to_string(),
            ),
            (
                socket_path
                    .to_str()
                    .expect("Socket path is invalid")
                    .to_string(),
                CONTAINER_BRIDGE_SOCKET.to_string(),
            ),
        ];

        let bridge_handle = tokio::spawn(client.run());

        let mut container_builder = image
            .create_dyn_container()
            .name(ulid.to_string())
            .network_mode(NetworkMode::Bridge)
            .add_volumes(volumes)
            .privileged(true) // Unfortunate hack for now
            .env("P2P_INDUSTRIES_SHARED_DIR", CONTAINER_SHARED_DIR)
            .env("P2P_INDUSTRIES_BRIDGE_SOCKET", CONTAINER_BRIDGE_SOCKET)
            .auto_remove(true);

        for port in ports {
            let ip4socket = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
            container_builder = container_builder.expose_port(port, ip4socket.into());
            let ip6socket = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);
            container_builder = container_builder.expose_port(port, ip6socket.into());
        }

        if let Some(shared_printer) = shared_printer {
            container_builder = container_builder
                .stdout(Box::new(shared_printer.clone()) as Box<_>)
                .stderr(Box::new(shared_printer) as Box<_>)
                .enable_stream();
        } else {
            container_builder = container_builder
                .stdout(Box::new(stdout()) as Box<_>)
                .stderr(Box::new(stderr()) as Box<_>)
                .enable_stream();
        }

        let running_container = container_builder.run().await?.into_owned();

        let (stop_sender, stop_receiver) = oneshot::channel();

        let handle = tokio::spawn(async move {
            let res = running_container
                .run_to_completion(Some(stop_receiver))
                .await;
            if let Err(e) = &res {
                tracing::error!(error = ?e, "Container exited with error");
            } else {
                tracing::debug!("Container exited");
            }

            cancellation_token.cancel();
            res.map_err(Into::into)
        });

        Ok(ContainerHandle::new(
            ulid,
            image_name,
            script_name,
            stop_sender,
            handle,
            bridge_handle,
        ))
    }
}

#[derive(Clone)]
pub struct ScriptingClient {
    client: P2PClient,
    container_manager: ContainerManager,
    self_command_sender: mpsc::Sender<SelfCommand>,
    exported_images: Arc<Mutex<HashMap<String, Cid>>>,
}

impl ScriptingClient {
    fn new(client: P2PClient, self_command_sender: mpsc::Sender<SelfCommand>) -> Self {
        Self {
            client,
            container_manager: ContainerManager::new().expect("Failed to create container manager"),
            self_command_sender,
            exported_images: Arc::default(),
        }
    }

    async fn get_image<'a>(
        &'a self,
        image: &'a str,
        local: bool,
        verbose: bool,
    ) -> Result<PulledImage<'a>, ExecutionError> {
        if local {
            Ok(self.container_manager.get_local_image(image))
        } else {
            self.container_manager
                .pull_image(image, verbose)
                .await
                .map_err(Into::into)
        }
    }

    pub async fn stop_all_containers(
        &self,
        kill: bool,
        peer_id: Option<PeerId>,
    ) -> Result<(), ExecutionError> {
        if let Some(peer_id) = peer_id {
            self.client
                .scripting()
                .stop_all_containers(peer_id, kill)
                .await
                .map_err(ExecutionError::StopContainerError)
        } else {
            let (sender, receiver) = oneshot::channel();
            let command = SelfCommand::StopAllContainers { kill, sender };

            self.self_command_sender
                .send(command)
                .await
                .map_err(|e| ExecutionError::StopContainerError(e.to_string()))?;

            receiver
                .await
                .map_err(|e| ExecutionError::StopContainerError(e.to_string()))?
        }
    }
}

impl bridge::ScriptingClient for ScriptingClient {
    type Error = ExecutionError;

    async fn deploy_image(
        &self,
        image: &str,
        local: bool,
        peer_id: PeerId,
        verbose: bool,
        ports: impl IntoIterator<Item = u16> + Send,
        persistent: bool,
    ) -> Result<Ulid, ExecutionError> {
        if peer_id == self.client.peer_id() {
            return self
                .self_deploy_image(image, local, verbose, ports, persistent)
                .await;
        }

        let pulled_image = self.get_image(image, local, verbose).await?;
        let image_id = pulled_image.get_id().await?;

        let cid = if let Some(cid) = OptionFuture::from(
            image_id.map(|id| async { self.exported_images.lock().await.get(id).copied() }),
        )
        .await
        .flatten()
        {
            cid
        } else {
            let image_archive = pulled_image.export(Compression::Zstd).await?;
            let escaped_image = image.replace('/', "_");
            let tmp = temp_dir().join(escaped_image);
            let mut file = File::create(&tmp).await?;
            file.write_all(&image_archive).await?;
            file.flush().await?;
            file.sync_data().await?;
            let cid = self.client.file_transfer().import_new_file(&tmp).await?;
            if let Some(id) = image_id {
                self.exported_images.lock().await.insert(id.into(), cid);
            }

            cid
        };

        let remote_ulid = self
            .client
            .scripting()
            .deploy_image(peer_id, cid, Compression::Zstd, ports, persistent)
            .await
            .map_err(ExecutionError::RemoteDeployError)?;
        Ok(remote_ulid)
    }

    async fn self_deploy_image(
        &self,
        image: &str,
        local: bool,
        verbose: bool,
        ports: impl IntoIterator<Item = u16> + Send,
        persistent: bool,
    ) -> Result<Ulid, ExecutionError> {
        let pulled_image = self.get_image(image, local, verbose).await?;
        let (sender, receiver) = oneshot::channel();
        let command = SelfCommand::DeployImage {
            image: pulled_image.into_owned(),
            ports: ports.into_iter().collect(),
            persistent,
            sender,
        };

        self.self_command_sender
            .send(command)
            .await
            .map_err(|e| ExecutionError::SelfDeployError(e.to_string()))?;

        receiver
            .await
            .map_err(|e| ExecutionError::SelfDeployError(e.to_string()))?
    }

    async fn list_containers(
        &self,
        peer_id: Option<PeerId>,
    ) -> Result<Vec<RunningScript>, ExecutionError> {
        if let Some(peer_id) = peer_id {
            self.client
                .scripting()
                .list_containers(peer_id)
                .await
                .map_err(ExecutionError::ListContainersError)
        } else {
            let (sender, receiver) = oneshot::channel();
            let command = SelfCommand::ListContainers { sender };

            self.self_command_sender
                .send(command)
                .await
                .map_err(|e| ExecutionError::ListContainersError(e.to_string()))?;

            receiver
                .await
                .map_err(|e| ExecutionError::ListContainersError(e.to_string()))
        }
    }

    async fn stop_container(
        &self,
        container_id: Ulid,
        peer_id: Option<PeerId>,
    ) -> Result<(), ExecutionError> {
        if let Some(peer_id) = peer_id {
            self.client
                .scripting()
                .stop_container(peer_id, container_id)
                .await
                .map_err(ExecutionError::StopContainerError)
        } else {
            let (sender, receiver) = oneshot::channel();
            let command = SelfCommand::StopContainer {
                container_id,
                sender,
            };

            self.self_command_sender
                .send(command)
                .await
                .map_err(|e| ExecutionError::StopContainerError(e.to_string()))?;

            receiver
                .await
                .map_err(|e| ExecutionError::StopContainerError(e.to_string()))?
        }
    }
}

struct ForbiddenScriptingClient;

impl bridge::ScriptingClient for ForbiddenScriptingClient {
    type Error = String;

    async fn deploy_image(
        &self,
        _image: &str,
        _local: bool,
        _peer: PeerId,
        _verbose: bool,
        _ports: impl IntoIterator<Item = u16> + Send,
        _persistent: bool,
    ) -> Result<Ulid, String> {
        Err("Scripting is not allowed".to_string())
    }

    async fn self_deploy_image(
        &self,
        _image: &str,
        _local: bool,
        _verbose: bool,
        _ports: impl IntoIterator<Item = u16> + Send,
        _persistent: bool,
    ) -> Result<Ulid, String> {
        Err("Scripting is not allowed".to_string())
    }

    async fn list_containers(
        &self,
        _peer_id: Option<PeerId>,
    ) -> Result<Vec<RunningScript>, String> {
        Err("Scripting is not allowed".to_string())
    }

    async fn stop_container(
        &self,
        _container_id: Ulid,
        _peer_id: Option<PeerId>,
    ) -> Result<(), String> {
        Err("Scripting is not allowed".to_string())
    }
}
