use std::{
    cmp::Ordering,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use colored::{ColoredString, Colorize};
use default_net::{get_interfaces, interface::InterfaceType, Interface};
use dialoguer::Select;
use futures::{future::join_all, stream::BoxStream};
use hyvectl_commands::families::init::Init;
use hyveos_sdk::Connection;
use tokio::{net::TcpSocket, time::timeout};

use crate::{error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

/// A helper struct to store relevant interface info.
#[derive(Debug)]
struct InterfaceInfo {
    name: String,
    is_ethernet: bool,
    is_connected: bool,
}

async fn is_internet_connected(iface: &Interface) -> bool {
    let check_addr = "8.8.8.8:53";

    for ip4_info in &iface.ipv4 {
        let local_ip = ip4_info.addr;
        let local_sock = SocketAddr::new(IpAddr::V4(local_ip), 0);

        let socket = match TcpSocket::new_v4() {
            Ok(socket) => socket,
            Err(_) => continue,
        };

        if socket.bind(local_sock).is_err() {
            continue;
        }

        let connect_future = socket.connect(check_addr.parse().expect("Invalid address"));
        match timeout(Duration::from_secs(1), connect_future).await {
            Ok(Ok(_)) => return true,
            _ => continue,
        }
    }

    false
}

async fn choose_bridge_network_interface() -> Option<String> {
    let interfaces = get_interfaces();

    let future_checks = interfaces.into_iter().map(|iface| async {
        let is_ethernet = matches!(
            iface.if_type,
            InterfaceType::Ethernet
                | InterfaceType::Ethernet3Megabit
                | InterfaceType::GigabitEthernet
        );

        let is_connected = is_internet_connected(&iface).await;

        InterfaceInfo {
            name: iface.name,
            is_ethernet,
            is_connected,
        }
    });

    let mut iface_info: Vec<InterfaceInfo> = join_all(future_checks).await;

    iface_info.sort_by(|a, b| match b.is_connected.cmp(&a.is_connected) {
        Ordering::Equal => match b.is_ethernet.cmp(&a.is_ethernet) {
            Ordering::Equal => a.name.cmp(&b.name),
            other => other,
        },
        other => match other {
            Ordering::Equal => match b.is_ethernet.cmp(&a.is_ethernet) {
                Ordering::Equal => a.name.cmp(&b.name),
                other => other,
            },
            other => other,
        },
    });

    let options: Vec<String> = iface_info
        .iter()
        .map(|info| {
            let status = if info.is_connected {
                "Online"
            } else {
                "Offline"
            };
            let eth = if info.is_ethernet {
                "Ethernet"
            } else {
                "Other"
            };
            format!("{} ({eth}, {status})", info.name)
        })
        .collect();

    todo!()
}

async fn choose_wirless_network_interface() -> Option<String> {
    fn if_type_score(ty: &InterfaceType) -> u8 {
        match ty {
            InterfaceType::Wireless80211 => 5,
            InterfaceType::Ethernet => 4,
            InterfaceType::Ethernet3Megabit => 3,
            InterfaceType::GigabitEthernet => 2,
            InterfaceType::Loopback => 0,
            _ => 1,
        }
    }

    let mut interfaces = get_interfaces();
    interfaces.sort_by_key(|iface| if_type_score(&iface.if_type));

    let options: Vec<String> = interfaces
        .iter()
        .map(|iface| {
            let is_up = if iface.is_up() {
                "↑ up".green()
            } else {
                "↓ down".yellow()
            };
            let is_loopback = if iface.is_loopback() {
                "↺ loopback".red()
            } else {
                ""
            };

            let ipv4_addrs: Vec<ColoredString> = iface
                .ipv4
                .iter()
                .map(|info| info.addr.to_string().bright_blue())
                .collect();
            let ipv6_addrs: Vec<ColoredString> = iface
                .ipv6
                .iter()
                .map(|info| info.addr.to_string().bright_cyan())
                .collect();
            let name = iface.name.white().on_black();
            format!("{name} {is_up} {is_loopback} {ipv4_addrs:?} {ipv6_addrs:?}")
        })
        .collect();

    if options.is_empty() {
        return None;
    }

    let selection = Select::new()
        .with_prompt("Choose a network interface to use for the mesh network")
        .items(&options)
        .default(0)
        .interact()
        .expect("Failed to get user input");
    interfaces.get(selection).map(|iface| iface.name.clone())
}

impl CommandFamily for Init {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        for interface in default_net::get_interfaces() {}

        todo!()
    }
}
