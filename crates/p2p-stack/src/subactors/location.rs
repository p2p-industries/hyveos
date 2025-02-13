use libp2p::{
    request_response::{cbor, Config, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

pub use self::{
    actor::{Actor, CommandError, EventError},
    command::Command,
};

mod actor;
mod client;
mod command;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    GetLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Location(Option<Location>),
}

pub type Behaviour = cbor::Behaviour<Request, Response>;
pub type Event = <Behaviour as NetworkBehaviour>::ToSwarm;

pub fn new() -> Behaviour {
    Behaviour::new(
        [(
            StreamProtocol::new("/location/1.0.0"),
            ProtocolSupport::Full,
        )],
        Config::default(),
    )
}
