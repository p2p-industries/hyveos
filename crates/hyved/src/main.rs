use std::{
    env,
    path::{Path, PathBuf},
};
#[cfg(feature = "network")]
use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use dirs::data_local_dir;
use hyveos_core::DAEMON_NAME;
use hyveos_ifaddr::{if_name_to_index, IfAddr};
use hyveos_runtime::{CliConnectionType, LogFilter, Runtime, RuntimeArgs, ScriptManagementConfig};
use libp2p::{
    identity::Keypair,
    multiaddr::{Multiaddr, Protocol},
};
use serde::Deserialize;

const LISTEN_PORT: u16 = 39811;

fn default_store_directory() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join(DAEMON_NAME)
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct Config {
    #[serde(default)]
    interfaces: Option<Vec<String>>,
    #[serde(default)]
    batman_interface: Option<String>,
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
    #[serde(default)]
    log_dir: Option<PathBuf>,
    #[serde(default)]
    log_level: LogFilter,
    #[serde(default)]
    cli_socket_path: Option<PathBuf>,
    #[cfg(feature = "network")]
    #[serde(default, deserialize_with = "deserialize_socket_addr")]
    cli_socket_addr: Option<SocketAddr>,
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
                let path = base_path.join(DAEMON_NAME).join("config.toml");
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

/// This daemon starts the HyveOS runtime.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Opts {
    #[clap(long)]
    pub config_file: Option<PathBuf>,
    #[clap(
        short,
        long = "listen-address",
        value_name = "LISTEN_ADDRESS",
        conflicts_with("interfaces")
    )]
    pub listen_addrs: Option<Vec<hyveos_ifaddr::IfAddr>>,
    #[clap(
        short,
        long = "interface",
        value_name = "INTERFACE",
        conflicts_with("listen_addrs")
    )]
    pub interfaces: Option<Vec<String>>,
    #[clap(
        long = "batman-address",
        value_name = "ADDRESS",
        conflicts_with("batman_interface")
    )]
    pub batman_addr: Option<hyveos_ifaddr::IfAddr>,
    #[clap(
        long = "batman-interface",
        value_name = "INTERFACE",
        conflicts_with("batman_addr")
    )]
    pub batman_interface: Option<String>,
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
    #[clap(short = 'o', long)]
    pub log_dir: Option<PathBuf>,
    #[clap(short = 'f', long)]
    pub log_level: Option<LogFilter>,
    #[clap(long, conflicts_with("cli_socket_addr"))]
    cli_socket_path: Option<PathBuf>,
    #[cfg(feature = "network")]
    #[clap(
        long,
        value_parser = parse_socket_addr,
        conflicts_with("cli_socket_path")
    )]
    cli_socket_addr: Option<SocketAddr>,
}

#[cfg(feature = "network")]
fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<Option<SocketAddr>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: Option<&str> = Deserialize::deserialize(deserializer)?;

    if let Some(s) = s {
        parse_socket_addr(s)
            .map_err(serde::de::Error::custom)
            .map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(feature = "network")]
fn parse_socket_addr(s: &str) -> Result<SocketAddr, anyhow::Error> {
    if let Some(s) = s.strip_prefix("[") {
        let mut parts = s.splitn(2, "]:");
        let host = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing host"))
            .and_then(|addr| IfAddr::from_str(addr).map_err(Into::into))?;
        let port = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing port"))
            .and_then(|port| port.parse().map_err(Into::into))?;

        host.with_port(port).map_err(Into::into)
    } else {
        s.parse().map_err(Into::into)
    }
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
    use hyveos_ifwatcher::{IfEvent, IfWatcher};

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
                        listen_addrs.push(if_addr.to_multiaddr(true));
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
                iface
                    .ipv6
                    .into_iter()
                    .map(move |net| IfAddr::new_with_index(net.addr(), iface.index))
            })
            .map(|res| res.map(|if_addr| if_addr.to_multiaddr(true)))
            .collect::<Result<_, _>>()?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Opts {
        config_file,
        listen_addrs,
        interfaces,
        batman_addr,
        batman_interface,
        store_directory,
        db_file,
        key_file,
        random_directory,
        script_management,
        clean,
        log_dir,
        log_level,
        cli_socket_path,
        #[cfg(feature = "network")]
        cli_socket_addr,
    } = Opts::parse();

    let Config {
        interfaces: config_interfaces,
        batman_interface: config_batman_interface,
        store_directory: config_store_directory,
        db_file: config_db_file,
        key_file: config_key_file,
        random_directory: config_random_directory,
        script_management: config_script_management,
        log_dir: config_log_dir,
        log_level: config_log_level,
        cli_socket_path: config_cli_socket_path,
        #[cfg(feature = "network")]
            cli_socket_addr: config_cli_socket_addr,
    } = Config::load(config_file)?;

    let listen_addrs = if let Some(addrs) = listen_addrs.map(|e| {
        e.into_iter()
            .map(|if_addr| if_addr.to_multiaddr(true))
            .collect()
    }) {
        addrs
    } else {
        fallback_listen_addrs(interfaces.or(config_interfaces)).await?
    };
    let listen_addrs = listen_addrs
        .into_iter()
        .map(|a| a.with(Protocol::Udp(LISTEN_PORT)).with(Protocol::QuicV1))
        .collect::<Vec<_>>();
    println!("Listen addresses: {listen_addrs:?}");

    let batman_addr = if let Some(addr) = batman_addr.map(|if_addr| if_addr.to_multiaddr(true)) {
        listen_addrs
            .iter()
            .find(|a| a.iter().next() == addr.iter().next())
            .cloned()
            .expect("batman_addr not in listen_addrs")
    } else {
        let batman_interface = batman_interface
            .or(config_batman_interface)
            .unwrap_or("bat0".to_string());

        let interface_index = if_name_to_index(batman_interface)?;

        listen_addrs
            .iter()
            .find(|&a| IfAddr::try_from(a).is_ok_and(|if_addr| if_addr.if_index == interface_index))
            .cloned()
            .expect("batman_interface not in listen_addrs")
    };

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

    let log_dir = log_dir.or(config_log_dir);
    let log_level = log_level.unwrap_or(config_log_level);

    #[cfg(feature = "network")]
    let cli_connection = if let Some(cli_connection) = cli_socket_path
        .map(CliConnectionType::Local)
        .or(cli_socket_addr.map(CliConnectionType::Network))
    {
        cli_connection
    } else if let Some(cli_socket_path) = config_cli_socket_path {
        if config_cli_socket_addr.is_some() {
            return Err(anyhow::anyhow!(
                "Config specifies both cli_socket_path and cli_socket_addr"
            ));
        }

        CliConnectionType::Local(cli_socket_path)
    } else if let Some(cli_socket_addr) = config_cli_socket_addr {
        CliConnectionType::Network(cli_socket_addr)
    } else {
        CliConnectionType::Local(
            hyveos_core::get_runtime_base_path()
                .join("bridge")
                .join("bridge.sock"),
        )
    };

    #[cfg(not(feature = "network"))]
    let cli_connection = CliConnectionType::Local(
        cli_socket_path
            .or(config_cli_socket_path)
            .unwrap_or_else(|| {
                hyveos_core::get_runtime_base_path()
                    .join("bridge")
                    .join("bridge.sock")
            }),
    );

    let args = RuntimeArgs {
        listen_addrs,
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
    };

    Runtime::new(args).await?.run().await
}
