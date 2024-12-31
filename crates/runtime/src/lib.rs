#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "network")]
use std::net::SocketAddr;
use std::{
    future::Future,
    path::{Path, PathBuf},
};

use futures::{future, FutureExt as _, TryFutureExt as _};
#[cfg(feature = "network")]
use hyveos_bridge::NetworkBridge;
use hyveos_bridge::{Bridge, ScriptingClient as _};
use hyveos_core::{get_runtime_base_path, gossipsub::ReceivedMessage};
#[cfg(feature = "batman")]
use hyveos_p2p_stack::DebugClient;
use hyveos_p2p_stack::{Client as P2PClient, FullActor};
use libp2p::{self, gossipsub::IdentTopic, identity::Keypair, Multiaddr};
pub use scripting::ScriptManagementConfig;
use tokio::task::{AbortHandle, JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::{
    db::Client as DbClient,
    scripting::{ScriptingClient, ScriptingManagerBuilder},
};

mod db;
mod future_map;
mod scripting;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum LogFilter {
    None,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogFilter> for LevelFilter {
    fn from(value: LogFilter) -> Self {
        match value {
            LogFilter::None => Self::OFF,
            LogFilter::Error => Self::ERROR,
            LogFilter::Warn => Self::WARN,
            LogFilter::Info => Self::INFO,
            LogFilter::Debug => Self::DEBUG,
            LogFilter::Trace => Self::TRACE,
        }
    }
}

#[derive(Debug)]
pub enum CliConnectionType {
    Local(PathBuf),
    #[cfg(feature = "network")]
    Network(SocketAddr),
}

#[derive(Clone)]
pub struct Clients {
    pub p2p_client: P2PClient,
    pub scripting_client: ScriptingClient,
}

#[derive(Debug)]
pub struct RuntimeArgs {
    pub listen_addrs: Vec<Multiaddr>,
    #[cfg(feature = "batman")]
    pub batman_addr: Multiaddr,
    pub store_directory: PathBuf,
    pub db_file: PathBuf,
    pub keypair: Keypair,
    pub random_directory: bool,
    pub script_management: ScriptManagementConfig,
    pub clean: bool,
    pub log_dir: Option<PathBuf>,
    pub log_level: LogFilter,
    pub cli_connection: CliConnectionType,
}

pub struct Runtime {
    clients: Clients,
    actor_task: JoinHandle<()>,
    file_provider_task: JoinHandle<()>,
    #[cfg(feature = "batman")]
    debug_client_task: JoinHandle<()>,
    scripting_manager_task: JoinHandle<()>,
    ping_task: JoinHandle<()>,
    cli_bridge_task: JoinHandle<Result<(), hyveos_bridge::Error>>,
    cli_bridge_cancellation_token: CancellationToken,
}

macro_rules! create_logfile {
    ($log_dir:ident, $log_level:ident) => {
        tracing_subscriber::fmt::layer()
            .with_timer(UtcTime::rfc_3339())
            .with_ansi(false)
            .with_writer(tracing_appender::rolling::minutely($log_dir, "runtime.log"))
            .with_filter(LevelFilter::from($log_level))
    };
}

macro_rules! create_printer {
    ($p:expr) => {
        tracing_subscriber::fmt::layer()
            .with_writer($p)
            .with_filter(EnvFilter::from_default_env())
    };
}

fn setup_logging(log_dir: Option<PathBuf>, log_level: LogFilter) {
    let registry = tracing_subscriber::registry().with(create_printer!(std::io::stdout));
    if let Some(log_dir) = log_dir {
        registry.with(create_logfile!(log_dir, log_level)).init();
    } else {
        registry.init();
    }
}

macro_rules! map_to_anyhow {
    ($e:ident) => {
        let $e = $e.map_err(anyhow::Error::from);
    };
}

impl Runtime {
    /// Create a new runtime that will listen on the given addresses
    ///
    /// # Errors
    ///
    /// TODO: Document errors
    #[allow(clippy::too_many_lines)]
    pub async fn new(args: RuntimeArgs) -> anyhow::Result<Runtime> {
        let RuntimeArgs {
            listen_addrs,
            #[cfg(feature = "batman")]
            batman_addr,
            store_directory,
            db_file,
            keypair,
            random_directory,
            script_management,
            clean,
            log_dir,
            log_level,
            cli_connection,
        } = args;

        setup_logging(log_dir, log_level);

        #[cfg(feature = "console-subscriber")]
        console_subscriber::init();

        if clean {
            Self::cleanup_store_directory(&store_directory)?;
        }

        let store_directory = if random_directory {
            store_directory.join(ulid::Ulid::new().to_string())
        } else {
            store_directory
        };

        println!("Store directory: {store_directory:?}");

        if !store_directory.exists() {
            std::fs::create_dir_all(&store_directory)?;
        }

        let runtime_base_path = get_runtime_base_path();

        let db_client = DbClient::new(db_file)?;

        let (p2p_client, mut actor) = FullActor::build(keypair);

        #[cfg(feature = "batman")]
        let opt_batman_addr = Some(batman_addr);
        #[cfg(not(feature = "batman"))]
        let opt_batman_addr = None;

        actor.setup(listen_addrs.into_iter(), opt_batman_addr);

        let actor_task = tokio::spawn(async move {
            tracing::trace!("Starting actor");
            Box::pin(actor.drive()).await;
        });

        let file_provider = p2p_client
            .file_transfer()
            .create_provider(store_directory.clone())
            .await
            .map_err(|_| anyhow::anyhow!("Failed to create file provider"))?;

        let file_provider_task = tokio::spawn(file_provider.run());

        #[cfg(feature = "batman")]
        let (debug_client, debug_command_sender) = DebugClient::build(p2p_client.clone());

        #[cfg(feature = "batman")]
        let debug_client_task = tokio::spawn(debug_client.run());

        let Ok(Some(scripting_command_broker)) = p2p_client.scripting().subscribe().await else {
            return Err(anyhow::anyhow!("Failed to get command broker"));
        };

        let builder = ScriptingManagerBuilder::new(
            scripting_command_broker,
            p2p_client.clone(),
            db_client.clone(),
            runtime_base_path.clone(),
            #[cfg(feature = "batman")]
            debug_command_sender.clone(),
            script_management,
        );

        let (scripting_manager, scripting_client) = builder.build();
        tracing::trace!("Starting scripting manager");
        let scripting_manager_task = tokio::spawn(scripting_manager.run());

        for (image, ports) in db_client.get_startup_scripts()? {
            tracing::trace!(?image, ?ports, "Deploying image");
            scripting_client
                .self_deploy_image(&image, true, false, ports, false)
                .await?;
        }

        let ping_task = tokio::spawn(Self::ping_task(p2p_client.clone()));

        let cli_bridge_base_path = runtime_base_path.join("bridge");
        let (cli_bridge_task, cli_bridge_cancellation_token) = match cli_connection {
            CliConnectionType::Local(socket_path) => {
                tracing::trace!(?socket_path, "Starting local bridge");
                let Bridge {
                    client,
                    cancellation_token,
                    ..
                } = Bridge::new(
                    p2p_client.clone(),
                    db_client,
                    cli_bridge_base_path,
                    socket_path,
                    #[cfg(feature = "batman")]
                    debug_command_sender,
                    scripting_client.clone(),
                )
                .await?;

                let task = tokio::spawn(client.run());
                (task, cancellation_token)
            }
            #[cfg(feature = "network")]
            CliConnectionType::Network(socket_addr) => {
                tracing::trace!(?socket_addr, "Starting network bridge");
                let NetworkBridge {
                    client,
                    cancellation_token,
                } = NetworkBridge::new(
                    p2p_client.clone(),
                    db_client,
                    cli_bridge_base_path,
                    socket_addr,
                    #[cfg(feature = "batman")]
                    debug_command_sender,
                    scripting_client.clone(),
                )
                .await?;

                let task = tokio::spawn(client.run());
                (task, cancellation_token)
            }
        };

        let clients = Clients {
            p2p_client,
            scripting_client,
        };

        Ok(Self {
            clients,
            actor_task,
            file_provider_task,
            #[cfg(feature = "batman")]
            debug_client_task,
            scripting_manager_task,
            ping_task,
            cli_bridge_task,
            cli_bridge_cancellation_token,
        })
    }

    /// Run the runtime until the p2p actor stops
    ///
    /// # Errors
    ///
    /// TODO: Document errors
    pub async fn run(self) -> anyhow::Result<()> {
        self.run_with(|_, _| future::ready(Ok(()))).await
    }

    /// Run the runtime and the given future in parallel
    ///
    /// The runtime will stop when the p2p actor stops or is aborted by the given future
    ///
    /// # Errors
    ///
    /// TODO: Document errors
    pub async fn run_with<'a, R>(
        self,
        f: impl FnOnce(Clients, AbortHandle) -> R,
    ) -> anyhow::Result<()>
    where
        R: Future<Output = anyhow::Result<()>> + 'a,
    {
        let Self {
            clients,
            actor_task,
            file_provider_task,
            #[cfg(feature = "batman")]
            debug_client_task,
            scripting_manager_task,
            ping_task,
            cli_bridge_task,
            cli_bridge_cancellation_token,
        } = self;

        let scripting_client = clients.scripting_client.clone();

        tokio::try_join!(
            f(clients, actor_task.abort_handle()).map_err(anyhow::Error::from),
            actor_task.map_err(anyhow::Error::from)
        )?;

        scripting_client.stop_all_containers(true, None).await?;

        file_provider_task.abort();
        #[cfg(feature = "batman")]
        debug_client_task.abort();
        scripting_manager_task.abort();
        ping_task.abort();

        cli_bridge_cancellation_token.cancel();

        map_to_anyhow!(file_provider_task);
        #[cfg(feature = "batman")]
        map_to_anyhow!(debug_client_task);
        map_to_anyhow!(scripting_manager_task);
        map_to_anyhow!(ping_task);

        let cli_bridge_task = cli_bridge_task.map(|res| match res {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(anyhow::Error::from(err)),
            Err(err) => Err(anyhow::Error::from(err)),
        });

        #[cfg(feature = "batman")]
        tokio::try_join!(
            file_provider_task,
            debug_client_task,
            scripting_manager_task,
            ping_task,
            cli_bridge_task,
        )?;

        #[cfg(not(feature = "batman"))]
        tokio::try_join!(
            file_provider_task,
            scripting_manager_task,
            ping_task,
            cli_bridge_task,
        )?;

        Ok(())
    }

    fn cleanup_store_directory(store_directory: impl AsRef<Path>) -> anyhow::Result<()> {
        let number_of_cleans: usize = store_directory
            .as_ref()
            .read_dir()
            .expect("Failed to read directory")
            .map(|e| {
                let entry = e?;
                if entry.file_type()?.is_dir() {
                    let m = entry
                        .path()
                        .file_name()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap();
                    let m = ulid::Ulid::from_string(&m)
                        .map(|_| {
                            std::fs::remove_dir_all(entry.path())?;
                            Ok::<_, std::io::Error>(1)
                        })
                        .unwrap_or(Ok(0usize))?;
                    return Ok(m);
                } else if entry.file_type()?.is_file() {
                    std::fs::remove_file(entry.path())?;
                    return Ok(1);
                }

                Ok::<usize, std::io::Error>(0)
            })
            .collect::<std::io::Result<Vec<usize>>>()?
            .into_iter()
            .sum();
        println!("Cleaned up {number_of_cleans} directories and files");

        Ok(())
    }

    async fn ping_task(client: P2PClient) {
        let gos = client.gossipsub();

        let topic_handle = gos.get_topic(IdentTopic::new("PING"));

        let mut receiver = topic_handle.subscribe().await.expect("Failed to subscribe");
        let round_trip = client.round_trip();

        loop {
            match receiver.recv().await {
                Ok(ReceivedMessage {
                    propagation_source,
                    source,
                    message_id,
                    message,
                }) => {
                    if let Some(source) = source {
                        if let Ok(nonce_data) = message.data.try_into() {
                            let nonce = u64::from_le_bytes(nonce_data);
                            tracing::debug!(
                                "Received pong message from {source} via \
                                {propagation_source} and message id: {message_id}"
                            );
                            round_trip.report_round_trip(source, nonce).await;
                        }
                    } else {
                        println!("Received pong message from unknown source");
                        continue;
                    }
                }
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
    }
}
