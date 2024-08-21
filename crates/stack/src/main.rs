#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "batman")]
use std::{collections::HashSet, sync::Arc};
use std::{
    env::temp_dir,
    fmt::Write as _,
    io::{self, IsTerminal as _},
    path::{Path, PathBuf},
};

use base64_simd::{Out, URL_SAFE};
use clap::Parser;
use dirs::{data_local_dir, runtime_dir};
use futures::stream::TryStreamExt as _;
use libp2p::{
    gossipsub::IdentTopic,
    identity::Keypair,
    kad::{
        BootstrapError, BootstrapOk, GetProvidersError, GetProvidersOk, GetRecordError,
        GetRecordOk, RecordKey,
    },
    multiaddr::Protocol,
    Multiaddr, PeerId,
};
use p2p_stack::{file_transfer::Cid, gossipsub::ReceivedMessage, Client, FullActor};
#[cfg(feature = "batman")]
use p2p_stack::{DebugClient, NeighbourEvent};
use rustyline::{error::ReadlineError, hint::Hinter, history::DefaultHistory, Editor};
use serde::Deserialize;
#[cfg(feature = "batman")]
use tokio::sync::broadcast::Receiver;
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt},
    task::JoinHandle,
    time::Instant,
};
use tracing_subscriber::EnvFilter;

use crate::{
    printer::SharedPrinter,
    scripting::{ScriptingClient, ScriptingManagerBuilder},
};

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
    key_file: Option<PathBuf>,
    #[serde(default)]
    random_directory: bool,
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
    #[clap(short, long)]
    pub key_file: Option<PathBuf>,
    #[clap(short, long)]
    pub random_directory: bool,
    #[clap(short, long)]
    pub clean: bool,
    #[clap(short, long)]
    pub vim: bool,
}

#[derive(rustyline::Completer, rustyline::Helper, rustyline::Validator, rustyline::Highlighter)]
struct DiyHinter {
    tree: patricia_tree::StringPatriciaMap<CommandHint>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl rustyline::hint::Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> Self {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.to_string(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DiyHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let line = line.to_uppercase();

        let line_str = line.as_str();

        let hint = self.tree.iter_prefix(line_str).find_map(|(_, hint)| {
            if hint.display.starts_with(line_str) {
                Some(hint.suffix(pos))
            } else {
                None
            }
        });

        hint
    }
}

fn diy_hints() -> DiyHinter {
    let mut tree = patricia_tree::StringPatriciaMap::new();
    tree.insert("ME", CommandHint::new("ME", "ME"));
    tree.insert(
        "SCRIPT HELP",
        CommandHint::new("SCRIPT HELP", "SCRIPT HELP"),
    );
    tree.insert("KAD HELP", CommandHint::new("KAD HELP", "KAD HELP"));
    tree.insert("KAD PUT", CommandHint::new("KAD PUT key value", "KAD PUT"));
    tree.insert("KAD GET", CommandHint::new("KAD GET key", "KAD GET"));
    tree.insert("KAD BOOT", CommandHint::new("KAD BOOT", "KAD BOOT"));
    tree.insert(
        "KAD PROVIDE",
        CommandHint::new("KAD PROVIDE key", "KAD PROVIDE"),
    );
    tree.insert(
        "KAD PROVIDERS",
        CommandHint::new("KAD PROVIDERS key", "KAD PROVIDERS"),
    );
    tree.insert("GOS HELP", CommandHint::new("GOS HELP", "GOS HELP"));
    tree.insert("GOS PING", CommandHint::new("GOS PING", "GOS PING"));
    tree.insert(
        "GOS PUB",
        CommandHint::new("GOS PUB topic message", "GOS PUB"),
    );
    tree.insert("GOS SUB", CommandHint::new("GOS SUB topic", "GOS SUB"));
    tree.insert("PING", CommandHint::new("PING", "PING"));
    #[cfg(feature = "batman")]
    {
        tree.insert("NEIGH HELP", CommandHint::new("NEIGH HELP", "NEIGH HELP"));
        tree.insert("NEIGH SUB", CommandHint::new("NEIGH SUB", "NEIGH SUB"));
        tree.insert("NEIGH LIST", CommandHint::new("NEIGH LIST", "NEIGH LIST"));
        tree.insert(
            "NEIGH LIST UNRESOLVED",
            CommandHint::new("NEIGH LIST UNRESOLVED", "NEIGH LIST UNRESOLVED"),
        );
    }
    tree.insert("FILE HELP", CommandHint::new("FILE HELP", "FILE HELP"));
    tree.insert("FILE LIST", CommandHint::new("FILE LIST", "FILE LIST"));
    tree.insert(
        "FILE IMPORT",
        CommandHint::new("FILE IMPORT path", "FILE IMPORT"),
    );
    tree.insert(
        "FILE GET",
        CommandHint::new("FILE GET ulid hash path", "FILE GET"),
    );
    tree.insert(
        "SCRIPT DEPLOY",
        CommandHint::new("SCRIPT DEPLOY SELF|peer_id image", "SCRIPT DEPLOY"),
    );
    tree.insert(
        "SCRIPT LOCAL DEPLOY",
        CommandHint::new(
            "SCRIPT LOCAL DEPLOY SELF|peer_id image",
            "SCRIPT LOCAL DEPLOY",
        ),
    );
    tree.insert(
        "SCRIPT LIST",
        CommandHint::new("SCRIPT LIST SELF|peer_id", "SCRIPT LIST"),
    );
    tree.insert(
        "SCRIPT STOP",
        CommandHint::new("SCRIPT STOP SELF|peer_id ALL|id", "SCRIPT STOP"),
    );
    tree.insert("QUIT", CommandHint::new("QUIT", "QUIT"));
    DiyHinter { tree }
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
    use std::time::Duration;

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
        key_file,
        random_directory,
        clean,
        vim,
    } = Opts::parse();

    let Config {
        interfaces: config_interfaces,
        store_directory: config_store_directory,
        key_file: config_key_file,
        random_directory: config_random_directory,
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

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        Box::pin(main_tty(
            listen_addrs,
            store_directory,
            keypair,
            random_directory,
            clean,
            vim,
        ))
        .await
    } else {
        main_alt(
            listen_addrs,
            store_directory,
            keypair,
            random_directory,
            clean,
        )
        .await
    }
}

#[allow(clippy::too_many_lines)]
async fn main_tty(
    listen_addrs: impl Iterator<Item = Multiaddr>,
    store_directory: PathBuf,
    keypair: Keypair,
    random_directory: bool,
    clean: bool,
    vim: bool,
) -> anyhow::Result<()> {
    let mut builder = rustyline::Config::builder();
    if vim {
        builder = builder.edit_mode(rustyline::EditMode::Vi);
    }

    let h = diy_hints();

    let mut rl: Editor<DiyHinter, DefaultHistory> = Editor::with_config(builder.build())?;
    rl.set_helper(Some(h));

    let rl_printer = SharedPrinter::from(rl.create_external_printer()?);

    let history_file = store_directory.join("history.txt");

    if history_file.exists() {
        rl.load_history(&history_file)?;
    } else {
        tokio::fs::File::create(&history_file).await?;
    }

    let Setup {
        client,
        scripting_client,
        actor_task,
        file_provider_task,
        #[cfg(feature = "batman")]
        debug_client_task,
        scripting_manager_task,
        ping_task,
    } = Setup::new(
        listen_addrs,
        store_directory,
        keypair,
        random_directory,
        clean,
        None,
    )
    .await?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str())?;
                if line.to_uppercase().starts_with("KAD") {
                    let kad = client.kad();
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, word)| {
                            if idx == 0 || idx == 1 {
                                word.to_uppercase()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<_>>();
                    let inner_split = split.iter().map(String::as_str).collect::<Vec<_>>();

                    match &inner_split[..] {
                        ["KAD", "HELP"] => {
                            help_message(&[
                                ("KAD PUT key value", "Put a record"),
                                ("KAD GET key", "Get a record"),
                                ("KAD BOOT", "Bootstrap the DHT"),
                                ("KAD PROVIDE key", "Start providing a record"),
                                ("KAD PROVIDERS key", "Get providers of a record"),
                            ]);
                        }
                        ["KAD", "PUT", key, value] => {
                            let res = kad
                                .put_record(
                                    RecordKey::new(&key),
                                    value.as_bytes().to_vec(),
                                    None,
                                    libp2p::kad::Quorum::One,
                                )
                                .await;
                            println!("Put Result: {res:?}");
                        }
                        ["KAD", "GET", key] => {
                            let res: Result<Vec<GetRecordOk>, GetRecordError> = kad
                                .get_record(RecordKey::new(&key))
                                .await
                                .expect("Failed to issue command")
                                .try_collect()
                                .await;
                            println!("Get Result: {res:?}");
                        }
                        ["KAD", "BOOT"] => {
                            let res: Result<Vec<BootstrapOk>, BootstrapError> = kad
                                .bootstrap()
                                .await
                                .expect("Failed to issue command")
                                .try_collect()
                                .await;
                            println!("Bootstrap Result: {res:?}");
                        }
                        ["KAD", "PROVIDE", key] => {
                            let res = kad.start_providing(RecordKey::new(&key)).await;
                            println!("Provide Result: {res:?}");
                        }
                        ["KAD", "PROVIDERS", key] => {
                            let res: Result<Vec<GetProvidersOk>, GetProvidersError> = kad
                                .get_providers(RecordKey::new(&key))
                                .await
                                .unwrap()
                                .try_collect()
                                .await;
                            println!("Providers Result: {res:?}");
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().starts_with("GOS") {
                    let gos = client.gossipsub();
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, word)| {
                            if idx == 0 || idx == 1 {
                                word.to_uppercase()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<_>>();
                    let inner_split = split.iter().map(String::as_str).collect::<Vec<_>>();

                    match &inner_split[..] {
                        ["GOS", "HELP"] => {
                            help_message(&[
                                ("GOS PUB topic message", "Publish a message"),
                                ("GOS SUB topic", "Subscribe to a topic"),
                                ("GOS PING", "Ping the network"),
                            ]);
                        }
                        ["GOS", "PING"] => {
                            let topic_handle = gos.get_topic(IdentTopic::new("PING"));
                            let nonce = rand::random();
                            let round_trip = client.round_trip();
                            let mut recv = round_trip.register(nonce).await;
                            let start = Instant::now();
                            let res = topic_handle.publish(nonce.to_le_bytes().to_vec()).await;
                            println!("Ping Send Result: {res:?}");
                            if res.is_err() {
                                continue;
                            }

                            let local_printer = rl_printer.clone();
                            tokio::spawn(async move {
                                let mut printer = local_printer;

                                loop {
                                    let msg = recv.recv().await.expect("Failed to receive");
                                    let from = msg.from;
                                    writeln!(
                                        printer,
                                        "Round trip to {from} ({nonce}) took {:?}",
                                        start.elapsed()
                                    )
                                    .expect("Failed to print");
                                }
                            });
                        }
                        ["GOS", "PUB", topic, message] => {
                            let topic_handle = gos.get_topic(IdentTopic::new(*topic));
                            let res = topic_handle.publish(message.as_bytes().to_vec()).await;
                            println!("Publish Result: {res:?}");
                        }
                        ["GOS", "SUB", topic] => {
                            let topic_handle = gos.get_topic(IdentTopic::new(*topic));
                            let mut res =
                                topic_handle.subscribe().await.expect("Failed to subscribe");

                            let local_printer = rl_printer.clone();
                            tokio::spawn(async move {
                                let mut printer = local_printer;

                                loop {
                                    match res.recv().await {
                                        Ok(msg) => {
                                            writeln!(printer, "Received: {msg:?}")
                                                .expect("Failed to print");
                                        }
                                        Err(err) => {
                                            writeln!(printer, "Error: {err:?}")
                                                .expect("Failed to print");
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().trim() == "PING" {
                    let ping = client.ping();
                    if let Ok(mut sub) = ping.subscribe().await {
                        let local_printer = rl_printer.clone();
                        tokio::spawn(async move {
                            let mut printer = local_printer;

                            while let Ok(event) = sub.recv().await {
                                let peer = event.peer;
                                if let Ok(dur) = event.result {
                                    writeln!(printer, "Ping to {peer} took {dur:?}")
                                        .expect("Failed to print");
                                } else {
                                    writeln!(printer, "Ping to {peer} failed")
                                        .expect("Failed to print");
                                }
                            }
                        });
                    } else {
                        println!("Failed to subscribe");
                    }
                } else if line.to_uppercase().starts_with("NEIGH") {
                    #[cfg(not(feature = "batman"))]
                    {
                        println!("batman feature not enabled");
                    }
                    #[cfg(feature = "batman")]
                    {
                        let neighbours = client.neighbours();
                        let split = line
                            .split_whitespace()
                            .map(str::to_uppercase)
                            .collect::<Vec<_>>();
                        let inner_split = split.iter().map(String::as_str).collect::<Vec<_>>();

                        match &inner_split[..] {
                            ["NEIGH", "HELP"] => {
                                help_message(&[
                                    ("NEIGH SUB", "Subscribe to neighbour events"),
                                    ("NEIGH LIST", "List resolved neighbours"),
                                    ("NEIGH LIST UNRESOLVED", "List unresolved neighbours"),
                                ]);
                            }
                            ["NEIGH", "SUB"] => {
                                if let Ok(sub) = neighbours.subscribe().await {
                                    tokio::spawn(neighbours_task(sub, rl_printer.clone()));
                                } else {
                                    println!("Failed to subscribe");
                                }
                            }
                            ["NEIGH", "LIST"] => {
                                if let Ok(neighbours) = neighbours.get_resolved().await {
                                    if neighbours.is_empty() {
                                        println!("No resolved neighbours");
                                    } else {
                                        println!("Resolved neighbours:");
                                        for (peer_id, neighbours) in neighbours {
                                            if let [neighbour] = &neighbours[..] {
                                                println!(
                                                    "  Peer {peer_id}: {}",
                                                    neighbour.direct_addr
                                                );
                                            } else {
                                                println!("  Peer {peer_id}:");
                                                for neighbour in neighbours {
                                                    println!("    - {}", neighbour.direct_addr);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    println!("Failed to get neighbours");
                                }
                            }
                            ["NEIGH", "LIST", "UNRESOLVED"] => {
                                if let Ok(neighbours) = neighbours.get_unresolved().await {
                                    if neighbours.is_empty() {
                                        println!("No unresolved neighbours");
                                    } else {
                                        println!("Unresolved neighbours:");
                                        for neighbour in neighbours {
                                            println!("  - {}", neighbour.mac);
                                        }
                                    }
                                } else {
                                    println!("Failed to get neighbours");
                                }
                            }
                            _ => {
                                println!("Invalid command");
                            }
                        }
                    }
                } else if line.to_uppercase().starts_with("SCRIPT") {
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, word)| {
                            if idx == 0 || idx == 1 {
                                word.to_uppercase()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<_>>();
                    let inner_split = split.iter().map(String::as_str).collect::<Vec<_>>();
                    match &inner_split[..] {
                        ["SCRIPT", "HELP"] => {
                            help_message(&[
                                (
                                    "SCRIPT DEPLOY SELF|peer_id image ports...",
                                    "Deploy a docker image to a peer, exposing the specified ports",
                                ),
                                (
                                    "SCRIPT LOCAL DEPLOY SELF|peer_id image ports...",
                                    "Deploy a local docker image to a peer, exposing the specified ports",
                                ),
                                (
                                    "SCRIPT LIST SELF|peer_id",
                                    "List all running scripts",
                                ),
                                (
                                    "SCRIPT STOP SELF|peer_id ALL|id",
                                    "Stop all running scripts or a specific script by ID",
                                ),
                            ]);
                        }
                        ["SCRIPT", "DEPLOY", self_, image, ports @ ..]
                            if self_.to_uppercase() == "SELF" =>
                        {
                            let ports = ports.iter().map(|e| e.parse().expect("Invalid port"));
                            match scripting_client
                                .self_deploy_image(image, false, true, ports)
                                .await
                            {
                                Ok(ulid) => {
                                    println!("Self-deployed image {image} with ULID {ulid}");
                                }
                                Err(e) => {
                                    println!("Failed to self-deploy image: {e:?}");
                                }
                            }
                        }
                        ["SCRIPT", "DEPLOY", peer_id, image, ports @ ..] => {
                            let peer: PeerId = peer_id.parse().expect("Invalid peer ID");
                            let ports = ports.iter().map(|e| e.parse().expect("Invalid port"));
                            match scripting_client
                                .deploy_image(image, false, peer, true, ports)
                                .await
                            {
                                Ok(ulid) => {
                                    println!(
                                        "Deployed image {image} to {peer_id} with ULID {ulid}"
                                    );
                                }
                                Err(e) => {
                                    println!("Failed to deploy image: {e:?}");
                                }
                            }
                        }
                        ["SCRIPT", "LOCAL", deploy, self_, image, ports @ ..]
                            if deploy.to_uppercase() == "DEPLOY"
                                && self_.to_uppercase() == "SELF" =>
                        {
                            let ports = ports.iter().map(|e| e.parse().expect("Invalid port"));
                            match scripting_client
                                .self_deploy_image(image, true, true, ports)
                                .await
                            {
                                Ok(ulid) => {
                                    println!("Self-deployed image {image} with ULID {ulid}");
                                }
                                Err(e) => {
                                    println!("Failed to self-deploy image: {e:?}");
                                }
                            }
                        }
                        ["SCRIPT", "LOCAL", deploy, peer_id, image, ports @ ..]
                            if deploy.to_uppercase() == "DEPLOY" =>
                        {
                            let peer: PeerId = peer_id.parse().expect("Invalid peer ID");
                            let ports = ports.iter().map(|e| e.parse().expect("Invalid port"));
                            match scripting_client
                                .deploy_image(image, true, peer, true, ports)
                                .await
                            {
                                Ok(ulid) => {
                                    println!(
                                        "Deployed image {image} to {peer_id} with ULID {ulid}"
                                    );
                                }
                                Err(e) => {
                                    println!("Failed to deploy image: {e:?}");
                                }
                            }
                        }
                        ["SCRIPT", "LIST", peer_id_or_self] => {
                            let peer_id = if peer_id_or_self.to_uppercase() == "SELF" {
                                None
                            } else {
                                Some(peer_id_or_self.parse().expect("Invalid peer ID"))
                            };

                            if let Ok(scripts) = scripting_client.list_containers(peer_id).await {
                                if scripts.is_empty() {
                                    println!("No running scripts");
                                } else {
                                    println!("Running scripts:");
                                    for (id, image) in scripts {
                                        println!("  {id}: {image}");
                                    }
                                }
                            } else {
                                println!("Failed to list scripts");
                            }
                        }
                        ["SCRIPT", "STOP", peer_id_or_self, all] if all.to_uppercase() == "ALL" => {
                            let peer_id = if peer_id_or_self.to_uppercase() == "SELF" {
                                None
                            } else {
                                Some(peer_id_or_self.parse().expect("Invalid peer ID"))
                            };

                            match scripting_client.stop_all_containers(false, peer_id).await {
                                Ok(()) => {
                                    println!("Stopped all containers");
                                }
                                Err(e) => {
                                    println!("Failed to stop all containers: {e}");
                                }
                            }
                        }
                        ["SCRIPT", "STOP", peer_id_or_self, id] => {
                            let peer_id = if peer_id_or_self.to_uppercase() == "SELF" {
                                None
                            } else {
                                Some(peer_id_or_self.parse().expect("Invalid peer ID"))
                            };

                            if let Ok(id) = id.parse() {
                                match scripting_client.stop_container(id, peer_id).await {
                                    Ok(()) => {
                                        println!("Stopped container with id {id}");
                                    }
                                    Err(e) => {
                                        println!("Failed to stop container: {e}");
                                    }
                                }
                            } else {
                                println!("Invalid ULID");
                            }
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().trim() == "ME" {
                    println!("Peer ID: {}", client.peer_id());
                    #[cfg(feature = "clipboard")]
                    {
                        arboard::Clipboard::new()
                            .expect("Failed to create clipboard")
                            .set_text(client.peer_id().to_string())
                            .expect("Failed to set clipboard");
                    }
                } else if line.to_uppercase().starts_with("FILE") {
                    let file_transfer = client.file_transfer();
                    let split = line
                        .split_whitespace()
                        .enumerate()
                        .map(|(idx, e)| {
                            if idx <= 1 {
                                e.to_uppercase()
                            } else {
                                e.to_string()
                            }
                        })
                        .collect::<Vec<_>>();
                    let inner_split = split.iter().map(String::as_str).collect::<Vec<_>>();

                    match &inner_split[..] {
                        ["FILE", "HELP"] => {
                            help_message(&[
                                ("FILE HELP", "Show this help"),
                                ("FILE LIST", "List files"),
                                ("FILE IMPORT path", "Import a file"),
                                ("FILE GET ulid hash path", "Get a file"),
                            ]);
                        }
                        ["FILE", "LIST"] => match file_transfer.list().await {
                            Ok(files) => {
                                let all_files = files.try_collect::<Vec<_>>().await;
                                match all_files {
                                    Ok(files) => {
                                        if files.is_empty() {
                                            println!("No files");
                                        } else {
                                            println!("Files:");
                                            for Cid { hash, id } in files {
                                                let hash_base =
                                                    URL_SAFE.encode_to_string(&hash[..]);
                                                println!("-  {id}: {hash_base}");
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("Failed to list files: {err:?}");
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to list files: {e:?}");
                            }
                        },
                        ["FILE", "IMPORT", path] => {
                            if let Ok(path) = path.parse::<PathBuf>() {
                                match file_transfer
                                    .import_new_file(
                                        path.as_path().canonicalize().unwrap().as_path(),
                                    )
                                    .await
                                {
                                    Err(e) => {
                                        println!("Failed to import file: {e:?}");
                                    }
                                    Ok(Cid { id, hash }) => {
                                        let hash_base = URL_SAFE.encode_to_string(&hash[..]);
                                        println!("Imported file with CID {id}: {hash_base}");
                                        #[cfg(feature = "clipboard")]
                                        {
                                            arboard::Clipboard::new()
                                                .expect("Failed to create clipboard")
                                                .set_text(format!("{id} {hash_base}"))
                                                .expect("Failed to set clipboard");
                                        }
                                    }
                                }
                            } else {
                                println!("Invalid path");
                            }
                        }
                        ["FILE", "GET", id, hash, path] => {
                            let cid_from_parts = || {
                                let id = id.parse().ok()?;
                                let mut hash_bytes = [0u8; 32];
                                let out = Out::from_slice(&mut hash_bytes[..]);
                                let hash_out = URL_SAFE.decode(hash.as_bytes(), out).ok()?;
                                if hash_out.len() == 32 {
                                    Some(Cid {
                                        id,
                                        hash: hash_bytes,
                                    })
                                } else {
                                    None
                                }
                            };
                            if let Some(cid) = cid_from_parts() {
                                match file_transfer.get_cid(cid).await {
                                    Ok(cid_path) => {
                                        let mut new_file = tokio::fs::File::create(path)
                                            .await
                                            .expect("Failed to create file");
                                        new_file.flush().await?;
                                        let mut file = tokio::fs::File::open(cid_path).await?;
                                        file.seek(std::io::SeekFrom::Start(0)).await.unwrap();
                                        tokio::io::copy(&mut file, &mut new_file)
                                            .await
                                            .expect("Failed to copy file");
                                    }
                                    Err(e) => {
                                        println!("Failed to get file: {e:?}");
                                    }
                                }
                            } else {
                                println!("Invalid CID");
                            }
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                } else if line.to_uppercase().trim() == "QUIT" {
                    break;
                } else {
                    println!("Invalid command");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }

    scripting_client.stop_all_containers(true, None).await?;

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

    rl.save_history(&history_file)?;

    Ok(())
}

#[cfg(feature = "batman")]
async fn neighbours_task(mut sub: Receiver<Arc<NeighbourEvent>>, mut printer: SharedPrinter) {
    while let Ok(event) = sub.recv().await {
        match event.as_ref() {
            NeighbourEvent::ResolvedNeighbour(neighbour) => {
                writeln!(
                    printer,
                    "Resolved neighbour {}: {}",
                    neighbour.peer_id, neighbour.direct_addr
                )
                .expect("Failed to print");
            }
            NeighbourEvent::LostNeighbour(neighbour) => {
                writeln!(
                    printer,
                    "Lost neighbour {}: {}",
                    neighbour.peer_id, neighbour.direct_addr
                )
                .expect("Failed to print");
            }
        }
    }
}

fn help_message(explains: &[(&str, &str)]) {
    let cmd_width_max = explains.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0) + 5;
    let explains_width_max = explains
        .iter()
        .map(|(_, explain)| explain.len())
        .max()
        .unwrap_or(0)
        + 5;
    println!("Available commands:");
    for (cmd, explain) in explains {
        println!("{cmd:<cmd_width_max$}: {explain:>explains_width_max$}");
    }
}

async fn main_alt(
    listen_addrs: impl Iterator<Item = Multiaddr>,
    store_directory: PathBuf,
    keypair: Keypair,
    random_directory: bool,
    clean: bool,
) -> anyhow::Result<()> {
    let Setup {
        scripting_client,
        actor_task,
        file_provider_task,
        #[cfg(feature = "batman")]
        debug_client_task,
        scripting_manager_task,
        ping_task,
        ..
    } = Setup::new(
        listen_addrs,
        store_directory,
        keypair,
        random_directory,
        clean,
        None,
    )
    .await?;

    actor_task.await?;

    scripting_client.stop_all_containers(true, None).await?;

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

fn clean_store(store_directory: &Path) -> anyhow::Result<()> {
    let number_of_cleans: usize = store_directory
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

struct Setup {
    client: Client,
    scripting_client: ScriptingClient,
    actor_task: JoinHandle<()>,
    file_provider_task: JoinHandle<()>,
    #[cfg(feature = "batman")]
    debug_client_task: JoinHandle<()>,
    scripting_manager_task: JoinHandle<()>,
    ping_task: JoinHandle<()>,
}

impl Setup {
    async fn new(
        listen_addrs: impl Iterator<Item = Multiaddr>,
        store_directory: PathBuf,
        keypair: Keypair,
        random_directory: bool,
        clean: bool,
        printer: Option<SharedPrinter>,
    ) -> anyhow::Result<Setup> {
        let subscriber = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env());

        if let Some(printer) = printer.as_ref() {
            subscriber.with_writer(printer.clone()).init();
        } else {
            subscriber.init();
        }

        #[cfg(feature = "console-subscriber")]
        console_subscriber::init();

        if clean {
            clean_store(&store_directory)?;
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

        let (client, mut actor) = FullActor::build(keypair);

        actor.setup(listen_addrs);

        let actor_task = tokio::spawn(async move {
            Box::pin(actor.drive()).await;
        });

        let file_provider = client
            .file_transfer()
            .create_provider(store_directory.clone())
            .await
            .map_err(|_| anyhow::anyhow!("Failed to create file provider"))?;

        let file_provider_task = tokio::spawn(file_provider.run());

        #[cfg(feature = "batman")]
        let (debug_client, debug_command_sender) = DebugClient::build(client.clone());

        #[cfg(feature = "batman")]
        let debug_client_task = tokio::spawn(debug_client.run());

        let Ok(Some(scripting_command_broker)) = client.scripting().subscribe().await else {
            return Err(anyhow::anyhow!("Failed to get command broker"));
        };

        let runtime_base_path =
            runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp").join(APP_NAME).join("runtime"));

        let mut builder = ScriptingManagerBuilder::new(
            scripting_command_broker,
            client.clone(),
            runtime_base_path,
            #[cfg(feature = "batman")]
            debug_command_sender,
        );

        if let Some(printer) = printer {
            builder = builder.with_printer(printer);
        }

        let (scripting_manager, scripting_client) = builder.build();
        let scripting_manager_task = tokio::spawn(scripting_manager.run());

        let ping_task = tokio::spawn(ping_task(client.clone()));

        Ok(Setup {
            client,
            scripting_client,
            actor_task,
            file_provider_task,
            #[cfg(feature = "batman")]
            debug_client_task,
            scripting_manager_task,
            ping_task,
        })
    }
}

async fn ping_task(client: Client) {
    let gos = client.gossipsub();

    let topic_handle = gos.get_topic(IdentTopic::new("PING"));

    let mut receiver = topic_handle.subscribe().await.expect("Failed to subscribe");
    let round_trip = client.round_trip();

    loop {
        match receiver.recv().await {
            Ok(ReceivedMessage {
                propagation_source,
                message_id,
                message,
            }) => {
                if let Some(source) = message.source {
                    if let Ok(nonce_data) = message.data.try_into() {
                        let nonce = u64::from_le_bytes(nonce_data);
                        tracing::debug!("Received pong message from {source} via {propagation_source} and message id: {message_id}");
                        round_trip.report_round_trip(source, nonce).await;
                    }
                } else {
                    println!("Received pong message from unkonwn source");
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
