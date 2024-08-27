#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{
    env::temp_dir,
    io::{self, IsTerminal as _},
    path::{Path, PathBuf},
};

use bridge::ScriptingClient as _;
use clap::Parser;
use dirs::{data_local_dir, runtime_dir};
use libp2p::{self, gossipsub::IdentTopic, identity::Keypair, multiaddr::Protocol, Multiaddr};
use p2p_industries_core::gossipsub::ReceivedMessage;
#[cfg(feature = "batman")]
use p2p_stack::DebugClient;
use p2p_stack::{Client as P2PClient, FullActor};
use scripting::ScriptManagementConfig;
use serde::Deserialize;
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;

use crate::{
    command_line::{interaction_loop, prep_interaction},
    db::Client as DbClient,
    printer::SharedPrinter,
    scripting::{ScriptingClient, ScriptingManagerBuilder},
};

mod command_line;
mod db;
mod future_map;
mod printer;
mod scripting;

const APP_NAME: &str = "p2p-industries-stack";

fn default_store_directory() -> PathBuf {
    data_local_dir().unwrap_or_else(temp_dir).join(APP_NAME)
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct Config {
    #[serde(default)]
    interfaces: Option<Vec<String>>,
    #[serde(default)]
    store_directory: Option<PathBuf>,
    #[serde(default)]
    db_file: Option<PathBuf>,
    #[serde(default)]
    key_file: Option<PathBuf>,
    #[serde(default)]
    random_directory: bool,
    #[serde(default)]
    script_management: Option<ScriptManagementConfig>,
}

impl Config {
    fn load(path: Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        if let Some(path) = path {
            std::fs::read_to_string(path)
                .map(|s| toml::from_str(&s))?
                .map_err(Into::into)
        } else {
            let base_paths = [Path::new("/etc"), Path::new("/usr/lib")];

            for base_path in &base_paths {
                let path = base_path.join(APP_NAME).join("config.toml");
                match std::fs::read_to_string(&path) {
                    Ok(s) => {
                        let config = toml::from_str(&s)?;
                        tracing::debug!("Loaded config file from {}", path.display());
                        return Ok(config);
                    }
                    Err(_) => {
                        tracing::info!("Failed to load config file from {}", path.display());
                    }
                }
            }

            Ok(Self::default())
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(long)]
    pub config_file: Option<PathBuf>,
    #[clap(
        short,
        long = "listen-address",
        value_name = "LISTEN_ADDRESS",
        conflicts_with("interfaces")
    )]
    pub listen_addrs: Option<Vec<ifaddr::IfAddr>>,
    #[clap(
        short,
        long = "interface",
        value_name = "INTERFACE",
        conflicts_with("listen_addrs")
    )]
    pub interfaces: Option<Vec<String>>,
    #[clap(short, long)]
    pub store_directory: Option<PathBuf>,
    #[clap(long)]
    pub db_file: Option<PathBuf>,
    #[clap(short, long)]
    pub key_file: Option<PathBuf>,
    #[clap(short, long)]
    pub random_directory: bool,
    #[clap(long, value_enum)]
    pub script_management: Option<ScriptManagementConfig>,
    #[clap(short, long)]
    pub clean: bool,
    #[clap(short, long)]
    pub vim: bool,
}

#[cfg(not(feature = "batman"))]
#[allow(clippy::unused_async)]
async fn fallback_listen_addrs(interfaces: Option<Vec<String>>) -> anyhow::Result<Vec<Multiaddr>> {
    use std::net::{Ipv4Addr, Ipv6Addr};

    if interfaces.is_some() {
        println!("Ignoring interfaces argument, batman feature is not enabled");
    }

    Ok([
        Multiaddr::empty().with(Protocol::Ip6(Ipv6Addr::UNSPECIFIED)),
        Multiaddr::empty().with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED)),
    ]
    .into_iter()
    .collect())
}

#[cfg(feature = "batman")]
async fn fallback_listen_addrs(interfaces: Option<Vec<String>>) -> anyhow::Result<Vec<Multiaddr>> {
    use std::{collections::HashSet, time::Duration};

    use futures::TryStreamExt as _;
    use ifaddr::{if_name_to_index, IfAddr};
    use ifwatcher::{IfEvent, IfWatcher};

    if let Some(mut interfaces) = interfaces
        .map(|i| {
            i.into_iter()
                .map(if_name_to_index)
                .collect::<Result<HashSet<_>, _>>()
        })
        .transpose()?
    {
        let get_interfaces = async move {
            let mut if_watcher = IfWatcher::new()?;
            let mut listen_addrs = Vec::new();

            while !interfaces.is_empty() {
                let Some(event) = if_watcher.try_next().await? else {
                    break;
                };

                if let IfEvent::Up(if_addr) = event {
                    if interfaces.remove(&if_addr.if_index) {
                        listen_addrs.push(Multiaddr::from(if_addr));
                    }
                }
            }

            Ok(listen_addrs)
        };

        tokio::time::timeout(Duration::from_secs(30), get_interfaces).await?
    } else {
        Ok(netdev::get_interfaces()
            .into_iter()
            .flat_map(|iface| {
                iface.ipv6.into_iter().map(move |net| IfAddr {
                    if_index: iface.index,
                    addr: net.addr,
                })
            })
            .map(Multiaddr::from)
            .collect())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Opts {
        config_file,
        listen_addrs,
        interfaces,
        store_directory,
        db_file,
        key_file,
        random_directory,
        script_management,
        clean,
        vim,
    } = Opts::parse();

    let Config {
        interfaces: config_interfaces,
        store_directory: config_store_directory,
        db_file: config_db_file,
        key_file: config_key_file,
        random_directory: config_random_directory,
        script_management: config_script_management,
    } = Config::load(config_file)?;

    let listen_addrs =
        if let Some(addrs) = listen_addrs.map(|e| e.into_iter().map(Multiaddr::from).collect()) {
            addrs
        } else {
            fallback_listen_addrs(interfaces.or(config_interfaces)).await?
        };
    println!("Listen addresses: {listen_addrs:?}");
    let listen_addrs = listen_addrs
        .into_iter()
        .map(|e| e.with(Protocol::Udp(0)).with(Protocol::QuicV1));

    let store_directory = store_directory
        .or(config_store_directory)
        .unwrap_or_else(default_store_directory);

    if !store_directory.exists() {
        std::fs::create_dir_all(&store_directory)?;
    }

    let db_file = db_file
        .or(config_db_file)
        .unwrap_or_else(|| store_directory.join("db"));

    let key_file = key_file
        .or(config_key_file)
        .unwrap_or_else(|| store_directory.join("keypair"));

    let keypair = if key_file.exists() {
        let protobuf_bytes = tokio::fs::read(key_file).await?;
        Keypair::from_protobuf_encoding(&protobuf_bytes)?
    } else {
        let keypair = Keypair::generate_ed25519();
        tokio::fs::write(&key_file, keypair.to_protobuf_encoding()?).await?;
        keypair
    };

    let random_directory = random_directory || config_random_directory;
    let script_management = script_management
        .or(config_script_management)
        .unwrap_or_default();

    let args = CommonArgs {
        listen_addrs,
        store_directory,
        db_file,
        keypair,
        random_directory,
        script_management,
        clean,
    };

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        Box::pin(main_tty(args, vim)).await
    } else {
        main_alt(args).await
    }
}

struct CommonArgs<Addrs> {
    listen_addrs: Addrs,
    store_directory: PathBuf,
    db_file: PathBuf,
    keypair: Keypair,
    random_directory: bool,
    script_management: ScriptManagementConfig,
    clean: bool,
}

async fn main_tty(
    args: CommonArgs<impl Iterator<Item = Multiaddr>>,
    vim: bool,
) -> anyhow::Result<()> {
    let (rl, rl_printer, history_file) = prep_interaction(&args.store_directory, vim).await?;

    let Setup {
        clients,
        actor_task,
        file_provider_task,
        #[cfg(feature = "batman")]
        debug_client_task,
        scripting_manager_task,
        ping_task,
    } = Setup::new(args, Some(rl_printer.clone())).await?;

    interaction_loop(rl, rl_printer, history_file, &clients).await?;

    clients
        .scripting_client
        .stop_all_containers(true, None)
        .await?;

    actor_task.abort();
    file_provider_task.abort();
    #[cfg(feature = "batman")]
    debug_client_task.abort();
    scripting_manager_task.abort();
    ping_task.abort();

    #[cfg(feature = "batman")]
    tokio::try_join!(
        actor_task,
        file_provider_task,
        debug_client_task,
        scripting_manager_task,
        ping_task,
    )?;

    #[cfg(not(feature = "batman"))]
    tokio::try_join!(
        actor_task,
        file_provider_task,
        scripting_manager_task,
        ping_task
    )?;

    Ok(())
}

async fn main_alt(args: CommonArgs<impl Iterator<Item = Multiaddr>>) -> anyhow::Result<()> {
    let Setup {
        clients,
        actor_task,
        file_provider_task,
        #[cfg(feature = "batman")]
        debug_client_task,
        scripting_manager_task,
        ping_task,
        ..
    } = Setup::new(args, None).await?;

    actor_task.await?;

    clients
        .scripting_client
        .stop_all_containers(true, None)
        .await?;

    file_provider_task.abort();
    #[cfg(feature = "batman")]
    debug_client_task.abort();
    scripting_manager_task.abort();
    ping_task.abort();

    #[cfg(feature = "batman")]
    tokio::try_join!(
        file_provider_task,
        debug_client_task,
        scripting_manager_task,
        ping_task,
    )?;

    #[cfg(not(feature = "batman"))]
    tokio::try_join!(file_provider_task, scripting_manager_task, ping_task)?;

    Ok(())
}

struct Clients {
    p2p_client: P2PClient,
    scripting_client: ScriptingClient,
}

struct Setup {
    clients: Clients,
    actor_task: JoinHandle<()>,
    file_provider_task: JoinHandle<()>,
    #[cfg(feature = "batman")]
    debug_client_task: JoinHandle<()>,
    scripting_manager_task: JoinHandle<()>,
    ping_task: JoinHandle<()>,
}

impl Setup {
    async fn new(
        args: CommonArgs<impl Iterator<Item = Multiaddr>>,
        printer: Option<SharedPrinter>,
    ) -> anyhow::Result<Setup> {
        let CommonArgs {
            listen_addrs,
            store_directory,
            db_file,
            keypair,
            random_directory,
            script_management,
            clean,
        } = args;

        let subscriber = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env());

        if let Some(printer) = printer.as_ref() {
            subscriber.with_writer(printer.clone()).init();
        } else {
            subscriber.init();
        }

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

        let db_client = DbClient::new(db_file)?;

        let (p2p_client, mut actor) = FullActor::build(keypair);

        actor.setup(listen_addrs);

        let actor_task = tokio::spawn(async move {
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

        let runtime_base_path =
            runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp").join(APP_NAME).join("runtime"));

        let mut builder = ScriptingManagerBuilder::new(
            scripting_command_broker,
            p2p_client.clone(),
            db_client.clone(),
            runtime_base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
            script_management,
        );

        if let Some(printer) = printer {
            builder = builder.with_printer(printer);
        }

        let (scripting_manager, scripting_client) = builder.build();
        let scripting_manager_task = tokio::spawn(scripting_manager.run());

        for (image, ports) in db_client.get_startup_scripts()? {
            scripting_client
                .self_deploy_image(&image, true, false, ports, false)
                .await?;
        }

        let ping_task = tokio::spawn(Self::ping_task(p2p_client.clone()));

        let clients = Clients {
            p2p_client,
            scripting_client,
        };

        Ok(Setup {
            clients,
            actor_task,
            file_provider_task,
            #[cfg(feature = "batman")]
            debug_client_task,
            scripting_manager_task,
            ping_task,
        })
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

                Ok::<usize, io::Error>(0)
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
