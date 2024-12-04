use futures::{Stream, StreamExt as _, TryStreamExt as _};
use libp2p_identity::PeerId;
use p2p_industries_core::{
    discovery::NeighbourEvent,
    grpc::{discovery_client::DiscoveryClient, Empty},
};
use tonic::transport::Channel;

use crate::{connection::P2PConnection, error::Result};

/// A handle to the discovery service.
///
/// Exposes methods to interact with the discovery service, such as subscribing to neighbour events
/// and getting the own peer ID.
///
/// # Example
///
/// ```no_run
/// use p2p_industries_sdk::P2PConnection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut discovery_service = connection.discovery();
/// let peer_id = discovery_service.get_own_id().await.unwrap();
///
/// println!("My peer id: {peer_id}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: DiscoveryClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &P2PConnection) -> Self {
        let client = DiscoveryClient::new(connection.channel.clone());

        Self { client }
    }

    /// Subscribes to neighbour events.
    ///
    /// Returns a stream of neighbour events. The stream will emit an event whenever the local
    /// runtime detects a change in the set of neighbours. The stream is guaranteed to emit an
    /// [`NeighbourEvent::Init`] directly after subscribing and only [`NeighbourEvent::Discovered`]
    /// and [`NeighbourEvent::Lost`] events afterwards.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails. The stream emits errors that occur in the runtime
    /// while processing the events, as well as data conversion errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::TryStreamExt as _;
    /// use p2p_industries_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let mut events = discovery_service.subscribe_events().await.unwrap();
    ///
    /// while let Some(event) = events.try_next().await.unwrap() {
    ///     println!("{event:?}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn subscribe_events(&mut self) -> Result<impl Stream<Item = Result<NeighbourEvent>>> {
        tracing::debug!("Received subscribe_events request");

        self.client
            .subscribe_events(Empty {})
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Returns the peer ID of the local runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_own_id(&mut self) -> Result<PeerId> {
        tracing::debug!("Received get_own_id request");

        self.client
            .get_own_id(Empty {})
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }
}
