#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]

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

pub mod grpc {
    #![allow(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]

    tonic::include_proto!("script");
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
