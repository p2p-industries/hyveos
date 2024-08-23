use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    discovery::NeighbourEvent,
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeshTopologyEvent {
    pub peer_id: PeerId,
    pub event: NeighbourEvent,
}

impl From<MeshTopologyEvent> for grpc::MeshTopologyEvent {
    fn from(event: MeshTopologyEvent) -> Self {
        Self {
            peer: event.peer_id.into(),
            event: event.event.into(),
        }
    }
}

impl TryFrom<grpc::MeshTopologyEvent> for MeshTopologyEvent {
    type Error = Error;

    fn try_from(event: grpc::MeshTopologyEvent) -> Result<Self> {
        Ok(Self {
            peer_id: event.peer.try_into()?,
            event: event.event.try_into()?,
        })
    }
}
