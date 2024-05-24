use tokio::sync::{mpsc, oneshot};

use super::{file_transfer, gossipsub, kad, ping, req_resp, round_trip};

#[cfg(feature = "batman")]
use super::debug;

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
    ReqResp(req_resp::Command),
    FileTransfer(file_transfer::Command),
    #[cfg(feature = "batman")]
    Debug(debug::Command),
}
