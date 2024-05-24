use libp2p::PeerId;
use tokio::sync::oneshot;

use super::Location;
use crate::impl_from_special_command;

/// Commands that the client can send to the actor.
#[derive(Debug)]
pub enum Command {
    #[allow(dead_code)]
    GetLocation {
        peer: PeerId,
        location: oneshot::Sender<Option<Location>>,
    },
}

impl_from_special_command!(Location);
