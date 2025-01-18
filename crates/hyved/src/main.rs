#[cfg(feature = "network")]
use std::net::SocketAddr;
use std::{env, path::PathBuf, time::Duration};

use clap::Parser;
use dirs::data_local_dir;
#[cfg(feature = "network")]
use hyveos_config::parse_socket_addr;
use hyveos_config::{ApplicationManagementConfig, Config, LogFilter};
use hyveos_core::DAEMON_NAME;
#[cfg(feature = "batman")]
use hyveos_ifaddr::if_name_to_index;
#[cfg(any(feature = "network", feature = "batman"))]
use hyveos_ifaddr::IfAddr;
use hyveos_runtime::{CliConnectionType, Runtime, RuntimeArgs};
use libp2p::{
    identity::Keypair,
    multiaddr::{Multiaddr, Protocol},
};

const LISTEN_PORT: u16 = 39811;

fn default_store_directory() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join(DAEMON_NAME)
}

/// This daemon starts and manages the hyveOS runtime.
///
/// Most of the command line arguments can also be set by editing the config file (see also `--config-file`).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Opts {
    /// Set the path to the configuration file (defaults to /etc/hyved/config.toml or /usr/lib/hyved/config.toml).
    #[clap(short, long, value_name = "FILE")]
    pub config_file: Option<PathBuf>,
    /// Set link-local interface addresses to listen on. Conflicts with `--interfaces`.
    ///
    /// Example: `fe80::1%wlan0,fe80::2%bat0`
    #[clap(
        short,
        long = "listen-addresses",
        value_name = "IF_ADDRESS,...",
        conflicts_with("interfaces")
    )]
    pub listen_addrs: Option<Vec<hyveos_ifaddr::IfAddr>>,
    /// Set interfaces to listen on. Conflicts with `--listen-addresses`.
    ///
    /// Example: `wlan0,bat0`
    #[clap(
        short,
        long,
        value_name = "INTERFACE,...",
        conflicts_with("listen_addrs")
    )]
    pub interfaces: Option<Vec<String>>,
    /// Set the link-local address of the batman interface. Conflicts with `--batman-interface`.
    ///
    /// Example: `fe80::1%bat0`
    #[cfg(feature = "batman")]
    #[clap(
        long = "batman-address",
        value_name = "IF_ADDRESS",
        conflicts_with("batman_interface")
    )]
    pub batman_addr: Option<hyveos_ifaddr::IfAddr>,
    /// Set the name of the batman interface. Conflicts with `--batman-address`.
    ///
    /// Example: `bat0`
    #[cfg(feature = "batman")]
    #[clap(long, value_name = "INTERFACE", conflicts_with("batman_addr"))]
    pub batman_interface: Option<String>,
    /// Set the directory to store runtime data in (defaults to $XDG_DATA_HOME/hyved or $HOME/.local/share/hyved).
    #[clap(short, long, value_name = "DIR")]
    pub store_directory: Option<PathBuf>,
    /// Set the path to the local database file (defaults to `store_directory`/db)
    #[clap(long, value_name = "FILE")]
    pub db_file: Option<PathBuf>,
    /// Set the path to the keypair file (defaults to `store_directory`/keypair)
    #[clap(short, long, value_name = "FILE")]
    pub key_file: Option<PathBuf>,
    /// Generate a random subdirectory in `store_directory` to store other runtime data in.
    #[clap(short, long)]
    pub random_directory: bool,
    /// Whether to enable application management by other running applications.
    #[clap(long, value_enum)]
    pub application_management: Option<ApplicationManagementConfig>,
    /// The application heartbeat timeout in seconds (defaults to 20).
    #[clap(long, value_name = "SECONDS")]
    pub application_heartbeat_timeout: Option<u64>,
    /// Clean the store directory on startup.
    #[clap(long)]
    pub clean: bool,
    /// Set the directory to store logs in. If not set, logs will only be printed to stdout.
    #[clap(short = 'o', long, value_name = "DIR")]
    pub log_dir: Option<PathBuf>,
    /// Set the log level filter (defaults to `info`).
    #[clap(short = 'f', long)]
    pub log_level: Option<LogFilter>,
    /// Set the path to the CLI socket file, used by `hyvectl` (defaults to /run/hyved/bridge/bridge.sock).
    /// Conflicts with `--cli-socket-addr`.
    #[cfg_attr(feature = "network", clap(conflicts_with("cli_socket_addr")))]
    #[clap(long, value_name = "FILE")]
    cli_socket_path: Option<PathBuf>,
    /// Set the network address of the CLI socket. Enables `hyvectl` connections over the network. Conflicts with `--cli-socket-path`.
    #[cfg(feature = "network")]
    #[clap(
        long,
        value_name = "NETWORK_ADDRESS",
        value_parser = parse_socket_addr,
        conflicts_with("cli_socket_path")
    )]
    cli_socket_addr: Option<SocketAddr>,
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
        #[cfg(feature = "batman")]
        batman_addr,
        #[cfg(feature = "batman")]
        batman_interface,
        store_directory,
        db_file,
        key_file,
        random_directory,
        application_management,
        application_heartbeat_timeout,
        clean,
        log_dir,
        log_level,
        cli_socket_path,
        #[cfg(feature = "network")]
        cli_socket_addr,
    } = Opts::parse();

    let Config {
        interfaces: config_interfaces,
        #[cfg(feature = "batman")]
            batman_interface: config_batman_interface,
        store_directory: config_store_directory,
        db_file: config_db_file,
        key_file: config_key_file,
        random_directory: config_random_directory,
        application_management: config_application_management,
        application_heartbeat_timeout: config_application_heartbeat_timeout,
        log_dir: config_log_dir,
        log_level: config_log_level,
        cli_socket_path: config_cli_socket_path,
        #[cfg(feature = "network")]
            cli_socket_addr: config_cli_socket_addr,
        telemetry,
        ..
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

    #[cfg(feature = "batman")]
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

    let apps_management = application_management
        .or(config_application_management)
        .unwrap_or_default();

    let application_heartbeat_timeout = Duration::from_secs(
        application_heartbeat_timeout
            .or(config_application_heartbeat_timeout)
            .unwrap_or(20),
    );

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
        #[cfg(feature = "batman")]
        batman_addr,
        store_directory,
        db_file,
        keypair,
        random_directory,
        apps_management,
        application_heartbeat_timeout,
        clean,
        log_dir,
        log_level,
        cli_connection,
        telemetry,
    };

    Runtime::new(args).await?.run().await
}
