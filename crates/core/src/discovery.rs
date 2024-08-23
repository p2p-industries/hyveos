use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NeighbourEvent {
    Init(Vec<PeerId>),
    Discovered(PeerId),
    Lost(PeerId),
}

impl From<NeighbourEvent> for grpc::NeighbourEvent {
    fn from(event: NeighbourEvent) -> Self {
        let event = match event {
            NeighbourEvent::Init(peer_ids) => {
                grpc::neighbour_event::Event::Init(grpc::Peers::from_iter(peer_ids))
            }
            NeighbourEvent::Discovered(peer_id) => {
                grpc::neighbour_event::Event::Discovered(peer_id.into())
            }
            NeighbourEvent::Lost(peer_id) => grpc::neighbour_event::Event::Lost(peer_id.into()),
        };

        Self { event: Some(event) }
    }
}

impl TryFrom<grpc::NeighbourEvent> for NeighbourEvent {
    type Error = Error;

    fn try_from(event: grpc::NeighbourEvent) -> Result<Self> {
        Ok(match event.event.ok_or(Error::MissingEvent)? {
            grpc::neighbour_event::Event::Init(peers) => NeighbourEvent::Init(peers.try_into()?),
            grpc::neighbour_event::Event::Discovered(peer) => {
                NeighbourEvent::Discovered(peer.try_into()?)
            }
            grpc::neighbour_event::Event::Lost(peer) => NeighbourEvent::Lost(peer.try_into()?),
        })
    }
}
