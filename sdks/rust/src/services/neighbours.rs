use futures::{Stream, StreamExt as _, TryStreamExt as _};
use hyveos_core::{
    grpc::{neighbours_client::NeighboursClient, Empty},
    neighbours::NeighbourEvent,
};
use libp2p_identity::PeerId;
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A handle to the neighbours service.
///
/// Exposes methods to interact with the neighbours service,
/// such as subscribing to neighbour events and getting the current set of neighbours.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut neighbours_service = connection.neighbours();
/// let neighbour_ids = neighbours_service.get().await.unwrap();
///
/// println!("Neighbours:");
/// for neighbour_id in neighbour_ids {
///    println!("- {neighbour_id}");
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: NeighboursClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = NeighboursClient::new(connection.channel.clone());

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
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut neighbours_service = connection.neighbours();
    /// let mut events = neighbours_service.subscribe().await.unwrap();
    ///
    /// while let Some(event) = events.try_next().await.unwrap() {
    ///     println!("{event:?}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn subscribe(&mut self) -> Result<impl Stream<Item = Result<NeighbourEvent>>> {
        self.client
            .subscribe(Empty {})
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Returns the peer IDs of the current neighbours.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut neighbours_service = connection.neighbours();
    /// let neighbour_ids = neighbours_service.get().await.unwrap();
    ///
    /// println!("Neighbours:");
    /// for neighbour_id in neighbour_ids {
    ///     println!("- {neighbour_id}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get(&mut self) -> Result<Vec<PeerId>> {
        self.client
            .get(Empty {})
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }
}
