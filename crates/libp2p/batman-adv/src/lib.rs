#![warn(clippy::expect_used, clippy::unwrap_used, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

use std::{sync::Arc, time::Duration};

use batman_neighbours_core::{BatmanNeighbour, Error as BatmanError};
use hyveos_ifaddr::if_name_to_index;
use libp2p::{Multiaddr, PeerId};
use macaddress::MacAddress;
use thiserror::Error;

pub use crate::behaviour::{store::ReadOnlyNeighbourStore, Behaviour, Event};

mod behaviour;

#[derive(Debug, Clone)]
pub struct Config {
    pub batman_if_index: u32,
    pub socket_path: Arc<str>,
    pub refresh_interval: Duration,
    pub neighbour_timeout: Duration,
    pub request_retries: u32,
    pub request_timeout: Duration,
}

impl Default for Config {
    #[allow(clippy::expect_used)] // We cannot implement Default otherwise
    fn default() -> Self {
        Self {
            batman_if_index: if_name_to_index("bat0").expect("Failed to resolve bat0"),
            socket_path: "/var/run/batman-neighbours.sock".into(),
            refresh_interval: Duration::from_secs(1),
            neighbour_timeout: Duration::from_secs(10),
            request_retries: 3,
            request_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnresolvedNeighbour {
    pub if_index: u32,
    pub mac: MacAddress,
}

impl From<BatmanNeighbour> for UnresolvedNeighbour {
    fn from(neighbour: BatmanNeighbour) -> Self {
        Self {
            if_index: neighbour.if_index,
            mac: neighbour.mac,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedNeighbour {
    pub peer_id: PeerId,
    pub if_index: u32,
    pub mac: MacAddress,
    pub batman_addr: Multiaddr,
    pub direct_addr: Multiaddr,
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Failed to discover neighbours: {0}")]
    BatmanError(#[from] BatmanError),
    #[error("{0} channel closed")]
    ChannelClosed(String),
    #[error("Failed to create netlink connection: {0}")]
    CreateNetlinkConnection(String),
    #[error("Failed to get ip addresses: {0}")]
    GetAddresses(String),
}
