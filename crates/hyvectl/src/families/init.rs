use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use colored::Colorize;
use default_net::{get_interfaces, interface::InterfaceType, Interface};
use dialoguer::{console::Term, Confirm, Select};
use futures::future::join_all;
use hyvectl_commands::families::init::Init;
use hyveos_config::Config;
use miette::Diagnostic;
use thiserror::Error;
use tokio::{net::TcpSocket, time::timeout};

use crate::{error::HyveCtlResult, out::CommandOutput};

fn if_type_to_str(ty: &InterfaceType) -> &'static str {
    match ty {
        InterfaceType::Unknown => "Unknown",
        InterfaceType::Ethernet => "Ethernet",
        InterfaceType::TokenRing => "Token Ring",
        InterfaceType::Fddi => "FDDI",
        InterfaceType::BasicIsdn => "Basic ISDN",
        InterfaceType::PrimaryIsdn => "Primary ISDN",
        InterfaceType::Ppp => "PPP",
        InterfaceType::Loopback => "Loopback",
        InterfaceType::Ethernet3Megabit => "3Mb Ethernet",
        InterfaceType::Slip => "SLIP",
        InterfaceType::Atm => "ATM",
        InterfaceType::GenericModem => "Generic Modem",
        InterfaceType::FastEthernetT => "Fast Ethernet (T)",
        InterfaceType::Isdn => "ISDN",
        InterfaceType::FastEthernetFx => "Fast Ethernet (FX)",
        InterfaceType::Wireless80211 => "Wireless 802.11 (WiFi) 🛜",
        InterfaceType::AsymmetricDsl => "Asymmetric DSL",
        InterfaceType::RateAdaptDsl => "Rate-Adapt DSL",
        InterfaceType::SymmetricDsl => "Symmetric DSL",
        InterfaceType::VeryHighSpeedDsl => "Very High Speed DSL",
        InterfaceType::IPOverAtm => "IP over ATM",
        InterfaceType::GigabitEthernet => "Gigabit Ethernet",
        InterfaceType::Tunnel => "Tunnel",
        InterfaceType::MultiRateSymmetricDsl => "Multi-Rate Symmetric DSL",
        InterfaceType::HighPerformanceSerialBus => "High Performance Serial Bus",
        InterfaceType::Wman => "WMAN",
        InterfaceType::Wwanpp => "WWAN PP",
        InterfaceType::Wwanpp2 => "WWAN PP2",
    }
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

async fn choose_wirless_network_interface() -> Result<Option<String>, Error> {
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

    let longest_name = interfaces
        .iter()
        .map(|iface| iface.name.len())
        .max()
        .unwrap_or(0);

    let longest_ty_str = interfaces
        .iter()
        .map(|iface| if_type_to_str(&iface.if_type).len())
        .max()
        .unwrap_or(0);

    let options: Vec<String> = join_all(interfaces
        .iter()
        .map(|iface| async move {
            let internet_connected = is_internet_connected(iface).await;
            let is_up = if iface.is_up() {
                "↑ up".green()
            } else {
                "↓ down".yellow()
            };
            let is_loopback = if iface.is_loopback() {
                "↺ loopback".red()
            } else {
                "".normal()
            };
            let internet = if internet_connected {
                "🌐 internet".green()
            } else {
                "no internet".red()
            };

            let ipv4_addrs: Vec<String> = iface
                .ipv4
                .iter()
                .map(|info| info.addr.to_string())
                .collect();
            let ipv6_addrs: Vec<String> = iface
                .ipv6
                .iter()
                .map(|info| info.addr.to_string())
                .collect();
            let name = iface.name.yellow().on_black();
            let ty_str = if_type_to_str(&iface.if_type).normal();
            format!(
                "{name:<longest_name$} {is_up:<5} {internet} {is_loopback:<10} {ty_str:<longest_ty_str$} {ipv4_addrs:?} {ipv6_addrs:?}"
            )
        }))
        .await;

    if options.is_empty() {
        return Ok(None);
    }

    let selection = Select::new()
        .with_prompt("Choose a network interface to use for the mesh network")
        .items(&options)
        .default(0)
        .interact_on(&Term::stderr())?;
    Ok(interfaces.get(selection).map(|iface| iface.name.clone()))
}

/// Errors by the init family
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    /// No wireless interface found
    #[error("No wireless interface found")]
    #[diagnostic(code(hyvectl::init::choose_wireless_interface::no_wireless_interface))]
    NoWirelessInterface,

    /// Dialoguer error
    #[error("Dialoguer error")]
    #[diagnostic(code(hyvectl::init::dialoguer_error))]
    Dialoguer(#[from] dialoguer::Error),

    /// Saving the config failed
    #[error("Saving the config failed")]
    #[diagnostic(code(hyvectl::init::save_config_failed))]
    SaveConfigFailed(anyhow::Error),
}

pub async fn init(_: Init) -> HyveCtlResult<CommandOutput> {
    let wifi_interface = choose_wirless_network_interface().await?;
    let wifi_interface = match wifi_interface {
        Some(wifi_interface) => {
            eprintln!("Wireless interface: {}", wifi_interface);
            wifi_interface
        }
        None => Err(Error::NoWirelessInterface)?,
    };

    let telemetry = Confirm::new()
        .with_prompt("Do you want to enable telemetry?")
        .default(true)
        .interact_on(&Term::stderr())
        .map_err(Error::Dialoguer)?;

    let config = Config {
        interfaces: Some(vec![wifi_interface.clone(), "bat0".to_string()]),
        wifi_interface: Some(wifi_interface.clone()),
        batman_interface: Some("bat0".to_string()),
        telemetry,
        ..Default::default()
    };

    config.save(None::<&str>).map_err(Error::SaveConfigFailed)?;

    Ok(CommandOutput::result()
        .with_field("wifi_interface", wifi_interface)
        .with_field("batman_interface", "bat0".to_string()))
}
