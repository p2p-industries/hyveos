#[cfg(feature = "batman")]
pub use self::client::TopicHandle;
pub use self::{
    actor::{Actor, CommandError, EventError},
    client::Client,
    command::Command,
};

mod actor;
mod client;
mod command;
