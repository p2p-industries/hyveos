mod actor;
mod behaviour;
mod client;
mod command;

pub mod file_transfer;
pub mod gossipsub;
mod identify;
pub mod kad;
pub mod ping;
pub mod req_resp;
pub mod round_trip;

#[cfg(feature = "mdns")]
pub mod mdns;

#[cfg(feature = "batman")]
pub mod neighbours;

#[cfg(feature = "location")]
pub mod location;

pub use self::{actor::Actor, client::Client};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::EventError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::EventError),
    #[cfg(feature = "location")]
    #[error("Location error: `{0}`")]
    Location(#[from] location::EventError),
}

impl From<void::Void> for EventError {
    fn from(inp: void::Void) -> Self {
        void::unreachable(inp)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::CommandError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::CommandError),
    #[cfg(feature = "location")]
    #[error("Location error: `{0}`")]
    Location(#[from] location::CommandError),
}

impl From<void::Void> for CommandError {
    fn from(inp: void::Void) -> Self {
        void::unreachable(inp)
    }
}

#[cfg(feature = "batman")]
type NeighbourActor = neighbours::Actor;
#[cfg(not(feature = "batman"))]
type NeighbourActor = ();

#[cfg(feature = "location")]
type LocationActor = location::Actor;
#[cfg(not(feature = "location"))]
type LocationActor = ();

#[cfg(feature = "mdns")]
type MdnsActor = mdns::Actor;
#[cfg(not(feature = "mdns"))]
type MdnsActor = ();

pub type FullActor = Actor<
    kad::Actor,
    MdnsActor,
    gossipsub::Actor,
    round_trip::Actor,
    LocationActor,
    ping::Actor,
    identify::Actor,
    NeighbourActor,
    req_resp::Actor,
    file_transfer::Actor,
    EventError,
    CommandError,
>;
