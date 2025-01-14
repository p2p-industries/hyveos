use futures::{Stream, StreamExt as _, TryStreamExt as _};
use hyveos_core::{
    dht::Key,
    grpc::{discovery_client::DiscoveryClient, DhtKey},
};
use libp2p_identity::PeerId;
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A handle to the discovery service.
///
/// Exposes methods to interact with the discovery service,
/// like for marking the local runtime as a provider for a discovery key
/// or getting the providers for a discovery key.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut discovery_service = connection.discovery();
/// discovery_service.provide("topic", "key").await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: DiscoveryClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = DiscoveryClient::new(connection.channel.clone());

        Self { client }
    }

    /// Marks the local runtime as a provider for a discovery key.
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
    /// let mut discovery_service = connection.discovery();
    /// discovery_service.provide("topic", "key").await.unwrap();
    /// # }
    /// ```
    #[tracing::instrument(skip_all, fields(topic))]
    pub async fn provide(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let key = Key {
            topic,
            key: key.into(),
        };

        self.client
            .provide(DhtKey::from(key))
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Gets the providers for a discovery key.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails. The stream emits errors that occur in the runtime
    /// while processing the providers, as well as data conversion errors.
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
    /// let mut discovery_service = connection.discovery();
    /// let mut providers = discovery_service.get_providers("topic", "key").await.unwrap();
    ///
    /// while let Some(provider) = providers.try_next().await.unwrap() {
    ///     println!("Found provider: {provider}");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip_all, fields(topic))]
    pub async fn get_providers(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<impl Stream<Item = Result<PeerId>>> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let key = Key {
            topic,
            key: key.into(),
        };

        self.client
            .get_providers(DhtKey::from(key))
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Stops providing a discovery key.
    ///
    /// Only affects the local node and only affects the network once the record expires.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn stop_providing(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<()> {
        self.client
            .stop_providing(DhtKey::from(Key {
                topic: topic.into(),
                key: key.into(),
            }))
            .await?;
        Ok(())
    }
}
