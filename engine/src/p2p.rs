mod actor;
mod behaviour;
mod client;
mod command;

pub mod gossipsub;
mod identify;
pub mod kad;
pub mod location;
pub mod mdns;
pub mod neighbours;
pub mod ping;
pub mod req_resp;
pub mod round_trip;

pub use {actor::Actor, client::Client};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::EventError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::EventError),
    #[error("Location error: `{0}`")]
    Location(#[from] location::EventError),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::CommandError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::CommandError),
    #[error("Location error: `{0}`")]
    Location(#[from] location::CommandError),
}

pub type FullActor = Actor<
    kad::Actor,
    mdns::Actor,
    gossipsub::Actor,
    round_trip::Actor,
    location::Actor,
    ping::Actor,
    identify::Actor,
    neighbours::Actor,
    req_resp::Actor,
    EventError,
    CommandError,
>;
