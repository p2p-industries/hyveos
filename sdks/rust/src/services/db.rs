use p2p_industries_core::grpc::{db_client::DbClient, DbKey, DbRecord};
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Channel;

use crate::{connection::P2PConnection, error::Result};

/// A handle to the database service.
///
/// Exposes methods to interact with the database service, like putting and getting key-value
/// records.
///
/// # Example
///
/// ```no_run
/// use p2p_industries_sdk::P2PConnection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let connection = P2PConnection::get().await.unwrap();
/// let mut db_service = connection.db();
/// assert!(db_service.put("key", b"value").await.unwrap().is_none());
///
/// let value = db_service.get("key").await.unwrap().unwrap();
/// assert_eq!(value, b"value");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: DbClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &P2PConnection) -> Self {
        let client = DbClient::new(connection.channel.clone());

        Self { client }
    }

    /// Puts a record into the key-value store.
    ///
    /// Returns the previous value if it exists, otherwise `None`.
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
    /// let mut db_service = connection.db();
    /// assert!(db_service.put("key", b"value").await.unwrap().is_none());
    ///
    /// let value = db_service.get("key").await.unwrap().unwrap();
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

        let request = DbRecord {
            key,
            value: value.into().into(),
        };

        let response = self.client.put(request).await?;

        Ok(response.into_inner().into())
    }

    /// Puts a record with a JSON-encoded value into the key-value store.
    ///
    /// Returns the previous value if it exists, otherwise `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or (de-)serialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut db_service = connection.db();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(db_service.put_json("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = db_service.get_json("key").await.unwrap().unwrap();
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
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or (de-)serialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut db_service = connection.db();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(db_service.put_cbor("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = db_service.get_cbor("key").await.unwrap().unwrap();
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
    /// let mut db_service = connection.db();
    /// assert!(db_service.put("key", b"value").await.unwrap().is_none());
    ///
    /// let value = db_service.get("key").await.unwrap().unwrap();
    /// assert_eq!(value, b"value");
    /// # }
    /// ```
    #[tracing::instrument(skip(self, key), fields(key))]
    pub async fn get(&mut self, key: impl Into<String>) -> Result<Option<Vec<u8>>> {
        let key = key.into();

        tracing::Span::current().record("key", &key);

        let response = self.client.get(DbKey { key }).await?;

        Ok(response.into_inner().into())
    }

    /// Gets a record with a JSON-encoded value from the key-value store.
    ///
    /// Returns the value if it exists, otherwise `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut db_service = connection.db();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(db_service.put_json("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = db_service.get_json("key").await.unwrap().unwrap();
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
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call or deserialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use p2p_industries_sdk::P2PConnection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Example {
    ///    value: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
    /// let mut db_service = connection.db();
    /// let value = Example { value: "Hello, world!".to_string() };
    /// assert!(db_service.put_cbor("key", &value).await.unwrap().is_none());
    ///
    /// let value: Example = db_service.get_cbor("key").await.unwrap().unwrap();
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