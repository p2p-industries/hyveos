use tokio::sync::{mpsc, oneshot};

#[cfg(feature = "location")]
use crate::subactors::location;
use crate::subactors::{apps, file_transfer, gossipsub, kad, ping, req_resp, round_trip};
#[cfg(feature = "batman")]
use crate::subactors::{debug, neighbours};

pub type SendResult<T, E> = oneshot::Sender<Result<T, E>>;
pub type RecvResult<T, E> = oneshot::Receiver<Result<T, E>>;

pub type SendMultipleResult<T, E> = mpsc::Sender<Result<T, E>>;

#[macro_export]
macro_rules! impl_from_special_command {
    ($spec:ident) => {
        impl From<Command> for $crate::command::Command {
            fn from(command: Command) -> Self {
                $crate::command::Command::$spec(command)
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
    Apps(apps::Command),
    FileTransfer(file_transfer::Command),
    #[cfg(feature = "batman")]
    Debug(debug::Command),
}
