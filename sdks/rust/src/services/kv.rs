use hyveos_core::{
    dht::Key,
    grpc::{kv_client::KvClient, Data, DhtKey, DhtRecord},
};
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A handle to the distributed key-value store service.
///
/// Exposes methods to interact with the key-value store service,
/// like for putting records into the key-value store or getting records from it.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut kv_service = connection.kv();
/// let value = kv_service.get_record("topic", "key").await.unwrap();
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
    client: KvClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = KvClient::new(connection.channel.clone());

        Self { client }
    }

    /// Puts a record into the key-value store.
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
    /// let mut kv_service = connection.kv();
    /// kv_service.put_record("topic", "key", "Hello, world!").await.unwrap();
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

        let request = DhtRecord {
            key: key.into(),
            value: Data { data: value.into() },
        };

        self.client
            .put_record(request)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Puts a record with a JSON-encoded value into the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut kv_service = connection.kv();
    /// let value = Example { message: "Hello, world!".to_string() };
    /// kv_service.put_record_json("topic", "key", &value).await.unwrap();
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

    /// Puts a record with a CBOR-encoded value into the key-value store.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut kv_service = connection.kv();
    /// let value = Example { message: "Hello, world!".to_string() };
    /// kv_service.put_record_cbor("topic", "key", &value).await.unwrap();
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

    /// Gets a record from the key-value store.
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
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut kv_service = connection.kv();
    /// let value = kv_service.get_record("topic", "key").await.unwrap();
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
            .map(|response| response.into_inner().data.map(|data| data.data))
            .map_err(Into::into)
    }

    /// Gets a record with a JSON-encoded value from the key-value store.
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
    /// use hyveos_sdk::Connection;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut kv_service = connection.kv();
    /// let value: Option<Example> = kv_service.get_record_json("topic", "key").await.unwrap();
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

    /// Gets a record with a CBOR-encoded value from the key-value store.
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
    /// use hyveos_sdk::Connection;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Example {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut kv_service = connection.kv();
    /// let value: Option<Example> = kv_service.get_record_cbor("topic", "key").await.unwrap();
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

    /// Removes a record from the key-value store.
    ///
    /// This only applies to the local node and only affects the network once the record expires.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    pub async fn remove_record(
        &mut self,
        topic: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<()> {
        self.remove_record(topic, key).await?;
        Ok(())
    }
}
