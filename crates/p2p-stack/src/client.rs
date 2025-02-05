use std::marker::PhantomData;

use libp2p::PeerId;
use tokio::sync::{mpsc, oneshot};

#[cfg(feature = "batman")]
use crate::subactors::{debug, neighbours};
use crate::{
    command::{Command, RecvResult},
    subactors::{apps, file_transfer, gossipsub, kad, ping, req_resp, round_trip},
};

#[derive(Clone)]
pub struct Client {
    peer_id: PeerId,
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub(super) fn new(sender: mpsc::Sender<Command>, peer_id: PeerId) -> Self {
        Self { peer_id, sender }
    }

    #[inline]
    fn special<C, T>(&self) -> T
    where
        T: From<SpecialClient<C>>,
        C: Into<Command>,
    {
        T::from(SpecialClient::<C> {
            sender: self.sender.clone(),
            peer_id: self.peer_id,
            _phantom: PhantomData,
        })
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn kad(&self) -> kad::Client {
        self.special()
    }

    pub fn gossipsub(&self) -> gossipsub::Client {
        self.special()
    }

    pub fn round_trip(&self) -> round_trip::Client {
        self.special()
    }

    pub fn ping(&self) -> ping::Client {
        self.special()
    }

    #[cfg(feature = "batman")]
    pub fn neighbours(&self) -> neighbours::Client {
        self.special()
    }

    pub fn req_resp(&self) -> req_resp::Client {
        self.special()
    }

    pub fn apps(&self) -> apps::Client {
        self.special()
    }

    pub fn file_transfer(&self) -> file_transfer::Client {
        self.special()
    }

    #[cfg(feature = "batman")]
    pub fn debug(&self) -> debug::Client {
        self.special()
    }
}

pub struct SpecialClient<T> {
    pub sender: mpsc::Sender<Command>,
    pub peer_id: PeerId,
    _phantom: PhantomData<T>,
}

impl<T> Clone for SpecialClient<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            peer_id: self.peer_id,
            _phantom: PhantomData,
        }
    }
}

impl<T> SpecialClient<T> {
    pub fn new(sender: mpsc::Sender<Command>, peer_id: PeerId) -> Self {
        Self {
            sender,
            peer_id,
            _phantom: PhantomData,
        }
    }
}

pub trait DynamicError: std::error::Error + Send + Sync + 'static {}

impl<T> DynamicError for T where T: std::error::Error + Send + Sync + 'static {}

#[derive(Debug, thiserror::Error)]
pub enum RequestError<E = &'static dyn DynamicError> {
    #[error("Failed to send command `{0}`")]
    Send(mpsc::error::SendError<Command>),
    #[error("Received error from command execution: `{0}`")]
    Recveived(#[from] E),
    #[error("Failed to receive response. This indicates that the actor pulling the libp2p stack has dropped the sender. `{0}`")]
    Oneshot(oneshot::error::RecvError),
}

pub type RequestResult<O, E> = Result<O, RequestError<E>>;

impl<T> SpecialClient<T>
where
    T: Into<Command>,
{
    pub async fn send(&self, special: T) -> Result<(), mpsc::error::SendError<Command>> {
        self.sender.send(special.into()).await
    }

    pub async fn request<O, E>(&self, special: T, recv: RecvResult<O, E>) -> RequestResult<O, E> {
        self.send(special).await.map_err(RequestError::Send)?;
        recv.await
            .map_err(RequestError::Oneshot)?
            .map_err(RequestError::Recveived)
    }
}
