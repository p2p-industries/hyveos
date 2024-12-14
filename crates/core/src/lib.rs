#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]

use std::{env, path::PathBuf};

use libp2p_identity::PeerId;
use ulid::Ulid;

pub use crate::error::{Error, Result};

pub mod debug;
pub mod dht;
pub mod discovery;
pub mod error;
pub mod file_transfer;
pub mod gossipsub;
pub mod req_resp;
#[doc(hidden)]
#[cfg(feature = "scripting")]
pub mod scripting;
#[doc(hidden)]
#[cfg(feature = "serde")]
pub mod serde;

pub mod grpc {
    #![allow(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]

    tonic::include_proto!("script");
}

pub const BRIDGE_SHARED_DIR_ENV_VAR: &str = "HYVEOS_BRIDGE_SHARED_DIR";
pub const BRIDGE_SOCKET_ENV_VAR: &str = "HYVEOS_BRIDGE_SOCKET";
pub const DAEMON_NAME: &str = "hyved";

#[must_use]
pub fn get_runtime_base_path() -> PathBuf {
    ["/run", "/var/run"]
        .into_iter()
        .map(str::to_string)
        .find_map(|s| PathBuf::from(s).canonicalize().ok())
        .unwrap_or_else(env::temp_dir)
        .join(DAEMON_NAME)
}

impl From<Vec<u8>> for grpc::Data {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<grpc::Data> for Vec<u8> {
    fn from(data: grpc::Data) -> Self {
        data.data
    }
}

impl From<Option<Vec<u8>>> for grpc::OptionalData {
    fn from(data: Option<Vec<u8>>) -> Self {
        Self {
            data: data.map(Into::into),
        }
    }
}

impl From<grpc::OptionalData> for Option<Vec<u8>> {
    fn from(data: grpc::OptionalData) -> Self {
        data.data.map(Into::into)
    }
}

impl From<PeerId> for grpc::Peer {
    fn from(peer_id: PeerId) -> Self {
        Self {
            peer_id: peer_id.to_string(),
        }
    }
}

impl TryFrom<grpc::Peer> for PeerId {
    type Error = Error;

    fn try_from(peer: grpc::Peer) -> Result<Self> {
        peer.peer_id.parse().map_err(Into::into)
    }
}

impl FromIterator<PeerId> for grpc::Peers {
    fn from_iter<I: IntoIterator<Item = PeerId>>(iter: I) -> Self {
        Self {
            peers: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<grpc::Peers> for Vec<PeerId> {
    type Error = Error;

    fn try_from(peers: grpc::Peers) -> Result<Self> {
        peers.peers.into_iter().map(TryInto::try_into).collect()
    }
}

impl From<Ulid> for grpc::Id {
    fn from(id: Ulid) -> Self {
        Self {
            ulid: id.to_string(),
        }
    }
}

impl TryFrom<grpc::Id> for Ulid {
    type Error = Error;

    fn try_from(id: grpc::Id) -> Result<Self> {
        id.ulid.parse().map_err(Into::into)
    }
}
