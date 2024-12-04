use futures::{Stream, StreamExt as _, TryStreamExt as _};
pub use hyveos_core::debug::{
    MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType, RequestDebugEvent,
    ResponseDebugEvent,
};
#[cfg(docsrs)]
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::grpc::{debug_client::DebugClient, Empty};
use tonic::transport::Channel;

use crate::{connection::P2PConnection, error::Result};

/// A handle to the debug service.
///
/// Exposes methods to interact with the debug service. Currently, the debug service only provides
/// a stream of mesh topology events, which are emitted whenever the mesh topology changes.
///
/// # Example
///
/// ```no_run
/// use futures::StreamExt as _;
/// use hyveos_sdk::P2PConnection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut debug_service = connection.debug();
/// let mut events = debug_service.subscribe_mesh_topology().await.unwrap();
///
/// while let Some(event) = events.next().await {
///     println!("{event:?}");
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: DebugClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &P2PConnection) -> Self {
        let client = DebugClient::new(connection.channel.clone());

        Self { client }
    }

    /// Subscribes to mesh topology events.
    ///
    /// Returns a stream of mesh topology events. The stream will emit an event whenever the mesh
    /// topology changes.
    ///
    /// For each peer in the mesh, it is guaranteed that the stream will first emit an event with a
    /// [`NeighbourEvent::Init`] when it enters the mesh, followed by only events with
    /// [`NeighbourEvent::Discovered`] or [`NeighbourEvent::Lost`], until the peer leaves the mesh.
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
    /// use hyveos_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut debug_service = connection.debug();
    /// let mut events = debug_service.subscribe_mesh_topology().await.unwrap();
    ///
    /// while let Some(event) = events.try_next().await.unwrap() {
    ///     println!("{event:?}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn subscribe_mesh_topology(
        &mut self,
    ) -> Result<impl Stream<Item = Result<MeshTopologyEvent>>> {
        self.client
            .subscribe_mesh_topology(Empty {})
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Subscribes to message debug events.
    ///
    /// Returns a stream of mesh debug events. The stream will emit an event whenever a request,
    /// response, or gossipsub message is sent by a peer in the mesh.
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
    /// use hyveos_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut debug_service = connection.debug();
    /// let mut events = debug_service.subscribe_messages().await.unwrap();
    ///
    /// while let Some(event) = events.try_next().await.unwrap() {
    ///     println!("{event:?}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn subscribe_messages(
        &mut self,
    ) -> Result<impl Stream<Item = Result<MessageDebugEvent>>> {
        self.client
            .subscribe_messages(Empty {})
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }
}
