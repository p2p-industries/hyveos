mod actor;
mod behaviour;
mod client;
mod command;

pub mod gossipsub;
pub mod kad;
pub mod mdns;
pub mod round_trip;

pub use actor::Actor;

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::EventError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::EventError),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Kad error: {0}")]
    Kad(#[from] kad::CommandError),
    #[error("Gossipsub error: `{0}`")]
    Gossipsub(#[from] gossipsub::CommandError),
}

pub type FullActor =
    Actor<kad::Actor, mdns::Actor, gossipsub::Actor, round_trip::Actor, EventError, CommandError>;
