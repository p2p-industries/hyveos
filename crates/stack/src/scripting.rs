use std::{
    collections::HashMap,
    env::temp_dir,
    future::Future,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
};

#[cfg(feature = "batman")]
use bridge::DebugCommandSender;
use bridge::{Bridge, CONTAINER_SHARED_DIR};
use bytes::Bytes;
use docker::{Compression, ContainerManager, NetworkMode, PulledImage, StoppedContainer};
use futures::{
    stream::{FuturesUnordered, TryStreamExt as _},
    FutureExt,
};
use libp2p::PeerId;
use p2p_stack::{
    file_transfer::{self, Cid},
    scripting::ActorToClient,
    Client as P2PClient,
};
use tokio::{
    fs::{metadata, File},
    io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use ulid::Ulid;

use crate::printer::SharedPrinter;

const CONTAINER_BRIDGE_SOCKET: &str = "/var/run/bridge.sock";

enum SelfCommand {
    DeployImage {
        image: PulledImage<'static>,
        ports: Vec<u16>,
        sender: oneshot::Sender<Result<Ulid, ExecutionError>>,
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
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
    shared_printer: Option<SharedPrinter>,
}

impl ScriptingManagerBuilder {
    pub fn new(
        command_broker: mpsc::Receiver<ActorToClient>,
        client: P2PClient,
        base_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
    ) -> Self {
        Self {
            command_broker,
            client,
            base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
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
            base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
            shared_printer,
        } = self;
        let (self_command_sender, self_command_receiver) = mpsc::channel(1);

        (
            ScriptingManager {
                command_broker,
                self_command_receiver,
                container_manager: ContainerManager::new()
                    .expect("Failed to create container manager"),
                client: client.clone(),
                base_path,
                #[cfg(feature = "batman")]
                debug_command_sender,
                shared_printer,
                container_handles: HashMap::new(),
            },
            ScriptingClient::new(client, self_command_sender),
        )
    }
}

pub struct ScriptingManager {
    command_broker: mpsc::Receiver<ActorToClient>,
    self_command_receiver: mpsc::Receiver<SelfCommand>,
    container_manager: ContainerManager,
    client: P2PClient,
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
    shared_printer: Option<SharedPrinter>,
    container_handles: HashMap<Ulid, ContainerHandle>,
}

impl ScriptingManager {
    fn execution_manager(&self) -> ExecutionManager {
        ExecutionManager {
            container_manger: &self.container_manager,
            client: self.client.clone(),
            base_path: self.base_path.clone(),
            #[cfg(feature = "batman")]
            debug_command_sender: self.debug_command_sender.clone(),
            shared_printer: self.shared_printer.clone(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(command) = self.command_broker.recv() => {
                    match command {
                        ActorToClient::DeployImage {
                            root_fs,
                            compression,
                            id: request_id,
                            ports,
                        } => {
                            let handle = self
                                .execution_manager()
                                .exec_foreign(root_fs, compression, ports)
                                .await;

                            let scripting = self.client.scripting().clone();

                            self.add_handle(handle, move |id| async move {
                                scripting
                                    .deployed_image(request_id, id.map_err(|e| e.to_string()))
                                    .map(|_| ())
                                    .await;
                            }).await;
                        },
                    }
                },
                Some(command) = self.self_command_receiver.recv() => {
                    match command {
                        SelfCommand::DeployImage {
                            image,
                            ports,
                            sender,
                        } => {
                            let handle = self
                                .execution_manager()
                                .exec(image, ports)
                                .await;

                            self.add_handle(handle, move |id| async move {
                                let _ = sender.send(id);
                            }).await;
                        }
                        SelfCommand::StopContainer {
                            container_id,
                            sender,
                        } => {
                            if let Some(handle) = self.container_handles.remove(&container_id) {
                                let _ = sender.send(handle.stop(false).await.and(Ok(())));
                            } else {
                                let _ = sender.send(Err(ExecutionError::ContainerNotFound(container_id)));
                            }
                        }
                        SelfCommand::StopAllContainers { kill, sender } => {
                            let containers = self.container_handles
                                .drain()
                                .map(|(_, handle)| handle.stop(kill))
                                .collect::<FuturesUnordered<_>>()
                                .try_collect::<Vec<_>>()
                                .await;

                            let result = match containers {
                                Ok(containers) => {
                                    tracing::info!("Stopped all containers: {containers:?}");
                                    Ok(())
                                },
                                Err(e) => Err(e),
                            };

                            let _ = sender.send(result);
                        }
                    }
                },
                else => { break }
            }
        }
    }

    async fn add_handle<Fut: Future<Output = ()>>(
        &mut self,
        handle: Result<ContainerHandle, ExecutionError>,
        send: impl FnOnce(Result<Ulid, ExecutionError>) -> Fut,
    ) {
        let id = match handle {
            Ok(handle) => {
                let id = handle.id;
                self.container_handles.insert(id, handle);
                Ok(id)
            }
            Err(e) => {
                tracing::error!("Failed to deploy image: {e}");
                Err(e)
            }
        };

        send(id).await;
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
    #[error("Container stop error: `{0}`")]
    StopContainerError(String),
}

struct ContainerHandle {
    id: Ulid,
    stop_sender: oneshot::Sender<bool>,
    handle: JoinHandle<Result<StoppedContainer<'static>, ExecutionError>>,
}

impl ContainerHandle {
    async fn stop(self, kill: bool) -> Result<Ulid, ExecutionError> {
        self.stop_sender
            .send(kill)
            .map_err(|e| ExecutionError::StopContainerError(e.to_string()))?;
        self.handle.await??;
        Ok(self.id)
    }
}

struct ExecutionManager<'a> {
    container_manger: &'a ContainerManager,
    client: P2PClient,
    base_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: DebugCommandSender,
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
        let Bridge {
            client,
            cancellation_token,
            ulid,
            socket_path,
            shared_dir_path,
        } = Bridge::new(
            self.client,
            self.base_path,
            #[cfg(feature = "batman")]
            self.debug_command_sender,
        )
        .await?;

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

        tokio::spawn(client.run());

        let mut container_builder = image
            .create_dyn_container()
            .name(ulid.to_string())
            .network_mode(NetworkMode::Bridge)
            .add_volumes(volumes)
            .env("P2P_INDUSTRIES_SHARED_DIR", CONTAINER_SHARED_DIR)
            .env("P2P_INDUSTRIES_BRIDGE_SOCKET", CONTAINER_BRIDGE_SOCKET);

        for port in ports {
            let ip4socket = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
            container_builder = container_builder.expose_port(port, ip4socket.into());
            let ip6socket = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);
            container_builder = container_builder.expose_port(port, ip6socket.into());
        }

        if let Some(shared_printer) = self.shared_printer {
            container_builder = container_builder
                .stdout(Box::new(shared_printer.clone()) as Box<_>)
                .stderr(Box::new(shared_printer) as Box<_>)
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

        Ok(ContainerHandle {
            id: ulid,
            stop_sender,
            handle,
        })
    }
}

#[derive(Clone)]
pub struct ScriptingClient {
    client: P2PClient,
    container_manager: ContainerManager,
    self_command_sender: mpsc::Sender<SelfCommand>,
}

impl ScriptingClient {
    fn new(client: P2PClient, self_command_sender: mpsc::Sender<SelfCommand>) -> Self {
        Self {
            client,
            container_manager: ContainerManager::new().expect("Failed to create container manager"),
            self_command_sender,
        }
    }

    pub async fn deploy_image(
        &self,
        image: &str,
        local: bool,
        peer: PeerId,
        verbose: bool,
        ports: impl IntoIterator<Item = u16>,
    ) -> Result<Ulid, ExecutionError> {
        let pulled_image = self.get_image(image, local, verbose).await?;
        let image_archive = pulled_image.export(Compression::Zstd).await?;
        let tmp = temp_dir().join(image);
        let mut file = File::create(&tmp).await?;
        file.write_all(&image_archive).await?;
        file.flush().await?;
        file.sync_data().await?;
        let cid = self.client.file_transfer().import_new_file(&tmp).await?;
        let remote_ulid = self
            .client
            .scripting()
            .deploy_image(peer, cid, Compression::Zstd, ports)
            .await
            .map_err(ExecutionError::RemoteDeployError)?;
        Ok(remote_ulid)
    }

    pub async fn self_deploy_image(
        &self,
        image: &str,
        local: bool,
        verbose: bool,
        ports: impl IntoIterator<Item = u16>,
    ) -> Result<Ulid, ExecutionError> {
        let pulled_image = self.get_image(image, local, verbose).await?;
        let (sender, receiver) = oneshot::channel();
        let command = SelfCommand::DeployImage {
            image: pulled_image.into_owned(),
            ports: ports.into_iter().collect(),
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

    pub async fn stop_container(&self, container_id: Ulid) -> Result<(), ExecutionError> {
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

    pub async fn stop_all_containers(&self, kill: bool) -> Result<(), ExecutionError> {
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
