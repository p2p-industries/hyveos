use tokio::sync::{mpsc, oneshot};

use super::{gossipsub, kad, ping, round_trip};

#[cfg(feature = "batman")]
use super::neighbours;

#[cfg(feature = "location")]
use super::location;

pub type SendResult<T, E> = oneshot::Sender<Result<T, E>>;
pub type RecvResult<T, E> = oneshot::Receiver<Result<T, E>>;

pub type SendMultipleResult<T, E> = mpsc::Sender<Result<T, E>>;

#[macro_export]
macro_rules! impl_from_special_command {
    ($spec:ident) => {
        impl From<Command> for $crate::p2p::command::Command {
            fn from(command: Command) -> Self {
                $crate::p2p::command::Command::$spec(command)
            }
        }
    };
}

pub enum Command {
    Kad(kad::Command),
    Gossipsub(gossipsub::Command),
    RoundTrip(round_trip::Command),
    #[cfg(feature = "location")]
    Location(location::Command),
    Ping(ping::Command),
    #[cfg(feature = "batman")]
    Neighbours(neighbours::Command),
}
