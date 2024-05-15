use tokio::sync::{mpsc, oneshot};

use super::{gossipsub, kad, location, neighbours, ping, req_resp, round_trip};

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
    Location(location::Command),
    Ping(ping::Command),
    Neighbours(neighbours::Command),
    ReqResp(req_resp::Command),
}
