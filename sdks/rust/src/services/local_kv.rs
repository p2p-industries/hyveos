use hyveos_core::grpc::{local_kv_client::LocalKvClient, LocalKvKey, LocalKvRecord};
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A handle to the local key-value store service.
///
/// Exposes methods to interact with the key-value store service,
/// like putting and getting key-value records.
/// The key-value store is local to the runtime and is not shared with other runtimes.
/// However, it is persisted across restarts of the runtime.
///
/// # Example
///
/// ```no_run
/// use hyveos_sdk::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = Connection::new().await.unwrap();
/// let mut local_kv_service = connection.local_kv();
/// assert!(local_kv_service.put("key", b"value").await.unwrap().is_none());
///
/// let value = local_kv_service.get("key").await.unwrap().unwrap();
/// assert_eq!(value, b"value");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: LocalKvClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = LocalKvClient::new(connection.channel.clone());

        Self { client }
    }

    /// Puts a record into the key-value store.
    ///
    /// Returns the previous value if it exists, otherwise `None`.
    /// This only has local effects and does not affect other runtimes.
    /// However, the record is persisted across restarts of the runtime.
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
    /// let mut local_kv_service = connection.local_kv();
    /// assert!(local_kv_service.put("key", b"value").await.unwrap().is_none());
    ///
    /// let value = local_kv_service.get("key").await.unwrap().unwrap();
    /// assert_eq!(value, b"value");
    /// # }
    /// ```
    #[tracing::instrument(skip(self, key, value), fields(key))]
    pub async fn put(
        &mut self,
        key: impl Into<String>,
        value: impl Into<Vec<u8>>,
    ) -> Result<Option<Vec<u8>>> {
        let key = key.into();

        tracing::Span::current().record("key", &key);

        let request = LocalKvRecord {
            key,
            value: value.into().into(),
        };

        let response = self.client.put(request).await?;

        Ok(response.into_inner().into())
    }

    /// Puts a record with a JSON-encoded value into the key-value store.
    ///
    /// Returns the previous value if it exists, otherwise `None`.
    /// This only has local effects and does not affect other runtimes.
    /// However, the record is persisted across restarts of the runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or (de-)serialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut local_kv_service = connection.local_kv();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(local_kv_service.put_json("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = local_kv_service.get_json("key").await.unwrap().unwrap();
    /// assert_eq!(value.value, "Hello, world!");
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn put_json<T: Serialize + DeserializeOwned>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<Option<T>> {
        let value = serde_json::to_vec(&value)?;

        self.put(key, value)
            .await?
            .map(|value| serde_json::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }

    /// Puts a record with a CBOR-encoded value into the key-value store.
    ///
    /// Returns the previous value if it exists, otherwise `None`.
    /// This only has local effects and does not affect other runtimes.
    /// However, the record is persisted across restarts of the runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or (de-)serialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut local_kv_service = connection.local_kv();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(local_kv_service.put_cbor("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = local_kv_service.get_cbor("key").await.unwrap().unwrap();
    /// assert_eq!(value.value, "Hello, world!");
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn put_cbor<T: Serialize + DeserializeOwned>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<Option<T>> {
        let value = serde_cbor::to_vec(&value)?;

        self.put(key, value)
            .await?
            .map(|value| serde_cbor::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }

    /// Gets a record from the key-value store if it exists.
    ///
    /// Returns the value if it exists, otherwise `None`.
    /// This will not return values from other runtimes.
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
    /// let mut local_kv_service = connection.local_kv();
    /// assert!(local_kv_service.put("key", b"value").await.unwrap().is_none());
    ///
    /// let value = local_kv_service.get("key").await.unwrap().unwrap();
    /// assert_eq!(value, b"value");
    /// # }
    /// ```
    #[tracing::instrument(skip(self, key), fields(key))]
    pub async fn get(&mut self, key: impl Into<String>) -> Result<Option<Vec<u8>>> {
        let key = key.into();

        tracing::Span::current().record("key", &key);

        let response = self.client.get(LocalKvKey { key }).await?;

        Ok(response.into_inner().into())
    }

    /// Gets a record with a JSON-encoded value from the key-value store.
    ///
    /// Returns the value if it exists, otherwise `None`.
    /// This will not return values from other runtimes.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut local_kv_service = connection.local_kv();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(local_kv_service.put_json("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = local_kv_service.get_json("key").await.unwrap().unwrap();
    /// assert_eq!(value.value, "Hello, world!");
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn get_json<T: DeserializeOwned>(
        &mut self,
        key: impl Into<String>,
    ) -> Result<Option<T>> {
        self.get(key)
            .await?
            .map(|value| serde_json::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }

    /// Gets a record with a CBOR-encoded value from the key-value store.
    ///
    /// Returns the value if it exists, otherwise `None`.
    /// This will not return values from other runtimes.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut local_kv_service = connection.local_kv();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(local_kv_service.put_cbor("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = local_kv_service.get_cbor("key").await.unwrap().unwrap();
    /// assert_eq!(value.value, "Hello, world!");
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn get_cbor<T: DeserializeOwned>(
        &mut self,
        key: impl Into<String>,
    ) -> Result<Option<T>> {
        self.get(key)
            .await?
            .map(|value| serde_cbor::from_slice(&value))
            .transpose()
            .map_err(Into::into)
    }
}
