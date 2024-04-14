use libp2p::PeerId;
use tokio::sync::oneshot;

use crate::impl_from_special_command;

use super::Location;

/// Commands that the client can send to the actor.
#[derive(Debug)]
pub enum Command {
    GetLocation {
        peer: PeerId,
        location: oneshot::Sender<Option<Location>>,
    },
}

impl_from_special_command!(Location);
