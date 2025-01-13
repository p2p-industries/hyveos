pub use crate::{actor::Actor, client::Client};
#[cfg(feature = "batman")]
pub use crate::{
    debug_client::{Command as DebugClientCommand, DebugClient},
    subactors::neighbours::Event as NeighbourEvent,
};

mod actor;
mod behaviour;
mod client;
mod command;
mod subactors;

#[cfg(feature = "batman")]
mod debug_client;

pub mod file_transfer {
    pub use crate::subactors::file_transfer::ClientError;
}

pub mod apps {
    pub use crate::subactors::apps::ActorToClient;
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Kad error: {0}")]
    Kad(#[from] subactors::kad::EventError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] subactors::gossipsub::EventError),
    #[cfg(feature = "location")]
    #[error("Location error: `{0}`")]
    Location(#[from] subactors::location::EventError),
}

impl From<void::Void> for EventError {
    fn from(inp: void::Void) -> Self {
        void::unreachable(inp)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Kad error: {0}")]
    Kad(#[from] subactors::kad::CommandError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] subactors::gossipsub::CommandError),
    #[cfg(feature = "location")]
    #[error("Location error: `{0}`")]
    Location(#[from] subactors::location::CommandError),
}

impl From<void::Void> for CommandError {
    fn from(inp: void::Void) -> Self {
        void::unreachable(inp)
    }
}

#[cfg(feature = "batman")]
type DebugActor = subactors::debug::Actor;
#[cfg(not(feature = "batman"))]
type DebugActor = ();

#[cfg(feature = "batman")]
type NeighbourActor = subactors::neighbours::Actor;
#[cfg(not(feature = "batman"))]
type NeighbourActor = ();

#[cfg(feature = "location")]
type LocationActor = subactors::location::Actor;
#[cfg(not(feature = "location"))]
type LocationActor = ();

#[cfg(feature = "mdns")]
type MdnsActor = subactors::mdns::Actor;
#[cfg(not(feature = "mdns"))]
type MdnsActor = ();

pub type FullActor = Actor<
    subactors::kad::Actor,
    MdnsActor,
    subactors::gossipsub::Actor,
    subactors::round_trip::Actor,
    LocationActor,
    subactors::ping::Actor,
    subactors::identify::Actor,
    NeighbourActor,
    subactors::req_resp::Actor,
    subactors::apps::Actor,
    subactors::file_transfer::Actor,
    DebugActor,
    EventError,
    CommandError,
>;
