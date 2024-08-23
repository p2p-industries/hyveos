use futures::{Stream, StreamExt as _, TryStreamExt as _};
use libp2p_identity::PeerId;
use p2p_industries_core::{
    dht::Key,
    grpc::{dht_client::DhtClient, DhtKey, DhtPutRecord},
};
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Channel;

use crate::{connection::P2PConnection, error::Result};

/// A handle to the DHT service.
///
/// Exposes methods to interact with the DHT service, like for putting and getting records, as well
/// as providing records.
///
/// # Example
///
/// ```no_run
/// use p2p_industries_sdk::P2PConnection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut dht_service = connection.dht();
/// let value = dht_service.get_record("topic", "key").await.unwrap();
///
/// if let Some(value) = value.and_then(|value| String::from_utf8(value).ok()) {
///     println!("Record has value: {value}");
/// } else {
///    println!("Record not found");
/// }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: DhtClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &P2PConnection) -> Self {
        let client = DhtClient::new(connection.channel.clone());

        Self { client }
    }

    /// Puts a record into the DHT.
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
    /// let mut dht_service = connection.dht();
    /// dht_service.put_record("topic", "key", "Hello, world!").await.unwrap();
    /// # }
    /// ```
    #[tracing::instrument(skip_all, fields(topic))]
    pub async fn put_record(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
    ) -> Result<()> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let key = Key {
            topic,
            key: key.into(),
        };

        let request = DhtPutRecord {
            key: key.into(),
            value: value.into(),
        };

        self.client
            .put_record(request)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Puts a record with a JSON-encoded value into the DHT.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let value = Example { message: "Hello, world!".to_string() };
    /// dht_service.put_record_json("topic", "key", &value).await.unwrap();
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn put_record_json(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
        value: &impl Serialize,
    ) -> Result<()> {
        let value = serde_json::to_vec(&value)?;

        self.put_record(topic, key, value).await
    }

    /// Puts a record with a CBOR-encoded value into the DHT.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let value = Example { message: "Hello, world!".to_string() };
    /// dht_service.put_record_cbor("topic", "key", &value).await.unwrap();
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn put_record_cbor(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
        value: &impl Serialize,
    ) -> Result<()> {
        let value = serde_cbor::to_vec(&value)?;

        self.put_record(topic, key, value).await
    }

    /// Gets a record from the DHT.
    ///
    /// Returns `None` if the record is not found.
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
    /// let mut dht_service = connection.dht();
    /// let value = dht_service.get_record("topic", "key").await.unwrap();
    ///
    /// if let Some(value) = value.and_then(|value| String::from_utf8(value).ok()) {
    ///     println!("Record has value: {value}");
    /// } else {
    ///    println!("Record not found");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip_all, fields(topic))]
    pub async fn get_record(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<Option<Vec<u8>>> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let key = Key {
            topic,
            key: key.into(),
        };

        self.client
            .get_record(DhtKey::from(key))
            .await
            .map(|response| response.into_inner().value)
            .map_err(Into::into)
    }

    /// Gets a record with a JSON-encoded value from the DHT.
    ///
    /// Returns `None` if the record is not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let value: Option<Example> = dht_service.get_record_json("topic", "key").await.unwrap();
    ///
    /// if let Some(value) = value {
    ///     println!("Record has value: {value:?}");
    /// } else {
    ///    println!("Record not found");
    /// }
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn get_record_json<T: DeserializeOwned>(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<Option<T>> {
        let value = self.get_record(topic, key).await?;

        value
            .map(|value| serde_json::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }

    /// Gets a record with a CBOR-encoded value from the DHT.
    ///
    /// Returns `None` if the record is not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let value: Option<Example> = dht_service.get_record_cbor("topic", "key").await.unwrap();
    ///
    /// if let Some(value) = value {
    ///     println!("Record has value: {value:?}");
    /// } else {
    ///    println!("Record not found");
    /// }
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn get_record_cbor<T: DeserializeOwned>(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<Option<T>> {
        let value = self.get_record(topic, key).await?;

        value
            .map(|value| serde_cbor::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }

    /// Marks the local stack as a provider for a key in the DHT.
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
    /// let mut dht_service = connection.dht();
    /// dht_service.provide("topic", "key").await.unwrap();
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

    /// Gets the providers for a key in the DHT.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails. The stream emits errors that occur in the stack
    /// while processing the providers, as well as data conversion errors.
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
    /// let mut dht_service = connection.dht();
    /// let mut providers = dht_service.get_providers("topic", "key").await.unwrap();
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
}
