use std::{env, path::PathBuf, sync::Arc};

use hyper_util::rt::TokioIo;
use hyveos_core::BRIDGE_SOCKET_ENV_VAR;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

#[cfg(feature = "cbor")]
use crate::services::CborReqRespService;
#[cfg(feature = "json")]
use crate::services::JsonReqRespService;
#[cfg(feature = "scripting")]
use crate::services::ScriptingService;
use crate::{
    error::{Error, Result},
    services::{
        DbService, DebugService, DhtService, DiscoveryService, FileTransferService,
        GossipSubService, ReqRespService,
    },
};

mod internal {
    use std::future::Future;

    use super::Connection;
    use crate::error::Result;

    pub trait ConnectionType {
        // We can promise `Send` here, so let's do it.
        fn connect(self) -> impl Future<Output = Result<Connection>> + Send;
    }
}

pub trait ConnectionType: internal::ConnectionType {}

impl<T: internal::ConnectionType> ConnectionType for T {}

/// A connection to the HyveOS runtime through the scripting bridge.
///
/// The Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
/// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
///
/// This is the standard connection type when used in a HyveOS script.
#[derive(Debug, Clone)]
pub struct BridgeConnection {
    _private: (),
}

impl internal::ConnectionType for BridgeConnection {
    async fn connect(self) -> Result<Connection> {
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(|_: Uri| async {
                let path = env::var(BRIDGE_SOCKET_ENV_VAR)
                    .map_err(|e| Error::EnvVarMissing(BRIDGE_SOCKET_ENV_VAR, e))?;

                UnixStream::connect(path)
                    .await
                    .map_err(Error::from)
                    .map(TokioIo::new)
            }))
            .await?;

        Ok(Connection {
            channel,
            #[cfg(feature = "network")]
            reqwest_client_and_url: None,
            use_bridge_shared_dir: true,
        })
    }
}

/// A connection to the HyveOS runtime through a custom Unix domain socket.
#[derive(Debug, Clone)]
pub struct CustomSocketConnection {
    socket_path: PathBuf,
}

impl internal::ConnectionType for CustomSocketConnection {
    async fn connect(self) -> Result<Connection> {
        let socket_path = Arc::new(self.socket_path);
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket_path = socket_path.clone();
                async move {
                    UnixStream::connect(socket_path.as_path())
                        .await
                        .map_err(Error::from)
                        .map(TokioIo::new)
                }
            }))
            .await?;

        Ok(Connection {
            channel,
            #[cfg(feature = "network")]
            reqwest_client_and_url: None,
            use_bridge_shared_dir: false,
        })
    }
}

/// A connection over the network to a HyveOS runtime listening at a given URI.
#[cfg(feature = "network")]
#[derive(Debug, Clone)]
pub struct UriConnection {
    uri: Uri,
}

#[cfg(feature = "network")]
impl internal::ConnectionType for UriConnection {
    async fn connect(self) -> Result<Connection> {
        let (url, if_name) = uri_to_url_and_if_name(self.uri.clone())?;
        let channel = Endpoint::from(self.uri).connect().await?;

        let mut client_builder = reqwest::Client::builder();

        if let Some(if_name) = if_name {
            client_builder = client_builder.interface(&if_name);
        }

        let client = client_builder.build()?;

        Ok(Connection {
            channel,
            reqwest_client_and_url: Some((client, url)),
            use_bridge_shared_dir: false,
        })
    }
}

#[cfg(feature = "network")]
fn uri_to_url_and_if_name(uri: Uri) -> Result<(reqwest::Url, Option<String>)> {
    let mut parts = uri.into_parts();
    let mut if_name = None;
    if let Some(authority) = &parts.authority {
        let authority = authority.as_str();

        if let Some(ipv6_start) = authority.find('[') {
            if let Some(start) = authority[ipv6_start..].find('%') {
                if let Some(end) = authority[start..].find(']') {
                    let zone = &authority[start + 1..end];
                    let name = zone
                        .parse()
                        .ok()
                        .and_then(|index| hyveos_ifaddr::if_index_to_name(index).ok())
                        .unwrap_or_else(|| zone.to_string());
                    if_name = Some(name);
                    let mut authority = authority.to_string();
                    authority.replace_range(start..end, "");
                    parts.authority = Some(authority.parse()?);
                }
            }
        }
    }

    Ok((Uri::try_from(parts)?.to_string().parse()?, if_name))
}

/// A builder for configuring a connection to the HyveOS runtime.
#[derive(Debug, Clone)]
pub struct ConnectionBuilder<T> {
    connection_type: T,
}

impl Default for ConnectionBuilder<BridgeConnection> {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionBuilder<BridgeConnection> {
    /// Creates a new builder for configuring a connection to the HyveOS runtime.
    ///
    /// By default, the connection to the HyveOS runtime will be made through the scripting bridge,
    /// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`custom_socket`] or [`uri`] methods.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_type: BridgeConnection { _private: () },
        }
    }

    /// Specifies a custom Unix domain socket to connect to.
    ///
    /// The socket path should point to a Unix domain socket that the HyveOS runtime is listening on.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::builder()
    ///     .custom_socket("/path/to/hyveos.sock")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub fn custom_socket(
        self,
        socket_path: impl Into<PathBuf>,
    ) -> ConnectionBuilder<CustomSocketConnection> {
        ConnectionBuilder {
            connection_type: CustomSocketConnection {
                socket_path: socket_path.into(),
            },
        }
    }

    /// Specifies a URI to connect to over the network.
    ///
    /// The URI should be in the format `http://<host>:<port>`.
    /// A HyveOS runtime should be listening at the given address.
    ///
    /// > **Note**: If the provided URI's path is not just `/` (e.g. `http://example.com:12345/foo/bar/`),
    /// > make sure that it ends with a slash!
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, Uri};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let uri = Uri::from_static("http://[::1]:50051");
    /// let connection = Connection::builder()
    ///     .uri(uri)
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[cfg(feature = "network")]
    pub fn uri(self, uri: Uri) -> ConnectionBuilder<UriConnection> {
        ConnectionBuilder {
            connection_type: UriConnection { uri },
        }
    }
}

impl<T: ConnectionType> ConnectionBuilder<T> {
    /// Establishes a connection to the HyveOS runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection could not be established.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::builder()
    ///     .custom_socket("/path/to/hyveos.sock")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub async fn connect(self) -> Result<Connection> {
        self.connection_type.connect().await
    }
}

/// A connection to the HyveOS runtime.
///
/// This struct provides access to the various services provided by HyveOS.
///
/// By default, the connection to the HyveOS runtime will be made through the scripting bridge,
/// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
/// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
/// If another connection type is desired, use the [`builder`] function to get a
/// [`ConnectionBuilder`] and use the [`ConnectionBuilder::custom_socket`] or
/// [`ConnectionBuilder::uri`] methods.
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
/// let peer_id = discovery_service.get_own_id().await.unwrap();
///
/// println!("My peer id: {peer_id}");
/// # }
/// ```
pub struct Connection {
    pub(crate) channel: Channel,
    #[cfg(feature = "network")]
    pub(crate) reqwest_client_and_url: Option<(reqwest::Client, reqwest::Url)>,
    pub(crate) use_bridge_shared_dir: bool,
}

impl Connection {
    /// Establishes a connection to the HyveOS runtime through the scripting bridge.
    ///
    /// The Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`builder`] function to get a
    /// [`ConnectionBuilder`] and use the [`ConnectionBuilder::custom_socket`] or
    /// [`ConnectionBuilder::uri`] methods.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection could not be established.
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
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        Connection::builder().connect().await
    }

    /// Creates a new builder for configuring a connection to the HyveOS runtime.
    ///
    /// By default, the connection to the HyveOS runtime will be made through the scripting bridge,
    /// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`ConnectionBuilder::custom_socket`] or
    /// [`ConnectionBuilder::uri`] methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::builder()
    ///     .custom_socket("/path/to/hyveos.sock")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> ConnectionBuilder<BridgeConnection> {
        ConnectionBuilder::new()
    }

    /// Returns a handle to the database service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut db_service = connection.db();
    /// assert!(db_service.put("key", b"value").await.unwrap().is_none());
    ///
    /// let value = db_service.get("key").await.unwrap().unwrap();
    /// assert_eq!(value, b"value");
    /// # }
    /// ```
    #[must_use]
    pub fn db(&self) -> DbService {
        DbService::new(self)
    }

    /// Returns a handle to the debug service.
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
    /// let mut debug_service = connection.debug();
    /// let mut events = debug_service.subscribe_mesh_topology().await.unwrap();
    ///
    /// while let Some(event) = events.try_next().await.unwrap() {
    ///     println!("{event:?}");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn debug(&self) -> DebugService {
        DebugService::new(self)
    }

    /// Returns a handle to the DHT service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
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
    #[must_use]
    pub fn dht(&self) -> DhtService {
        DhtService::new(self)
    }

    /// Returns a handle to the discovery service.
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
    /// let peer_id = discovery_service.get_own_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn discovery(&self) -> DiscoveryService {
        DiscoveryService::new(self)
    }

    /// Returns a handle to the file transfer service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let shared_dir = std::env::var(hyveos_core::BRIDGE_SHARED_DIR_ENV_VAR).unwrap();
    /// let file_path = Path::new(&shared_dir).join("example.txt");
    /// tokio::fs::write(&file_path, "Hello, world!").await.unwrap();
    ///
    /// let connection = Connection::new().await.unwrap();
    /// let mut file_transfer_service = connection.file_transfer();
    /// let cid = file_transfer_service.publish_file(&file_path).await.unwrap();
    ///
    /// println!("Content ID: {cid:?}");
    /// # }
    /// ```
    #[must_use]
    pub fn file_transfer(&self) -> FileTransferService {
        FileTransferService::new(self)
    }

    /// Returns a handle to the gossipsub service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut gossipsub_service = connection.gossipsub();
    /// let id = gossipsub_service.publish("topic", "Hello, world!").await.unwrap();
    ///
    /// println!("Published message with id: {id}");
    /// # }
    /// ```
    #[must_use]
    pub fn gossipsub(&self) -> GossipSubService {
        GossipSubService::new(self)
    }

    /// Returns a handle to the request-response service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let peer_id = dht_service
    ///     .get_providers("identification", "example")
    ///     .await
    ///     .unwrap()
    ///     .next()
    ///     .await
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// let mut req_resp_service = connection.req_resp();
    /// let response = req_resp_service
    ///     .send_request(peer_id, "Hello, world!", None)
    ///     .await
    ///     .unwrap();
    ///
    /// let data = Result::from(response).unwrap();
    /// println!("Received response: {}", String::from_utf8(data).unwrap());
    /// # }
    /// ```
    #[must_use]
    pub fn req_resp(&self) -> ReqRespService {
        ReqRespService::new(self)
    }

    /// Returns a handle to the request-response service with JSON-encoded requests and responses.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use hyveos_sdk::Connection;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct ExampleRequest {
    ///    message: String,
    /// }
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct ExampleResponse {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let peer_id = dht_service
    ///     .get_providers("identification", "example")
    ///     .await
    ///     .unwrap()
    ///     .next()
    ///     .await
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// let mut req_resp_service = connection.req_resp_json();
    /// let request = ExampleRequest { message: "Hello, world!".to_string() };
    /// let response = req_resp_service
    ///     .send_request(peer_id, &request, None)
    ///     .await
    ///     .unwrap();
    ///
    /// let data: ExampleResponse = Result::from(response).unwrap();
    /// println!("Received response: {data:?}");
    /// # }
    /// ```
    #[cfg(feature = "json")]
    #[must_use]
    pub fn req_resp_json<Req, Resp>(&self) -> JsonReqRespService<Req, Resp>
    where
        Req: Serialize + DeserializeOwned,
        Resp: Serialize + DeserializeOwned,
    {
        JsonReqRespService::new(self)
    }

    /// Returns a handle to the request-response service with JSON-encoded requests and responses.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use hyveos_sdk::Connection;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct ExampleRequest {
    ///    message: String,
    /// }
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct ExampleResponse {
    ///    message: String,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let peer_id = dht_service
    ///     .get_providers("identification", "example")
    ///     .await
    ///     .unwrap()
    ///     .next()
    ///     .await
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// let mut req_resp_service = connection.req_resp_cbor();
    /// let request = ExampleRequest { message: "Hello, world!".to_string() };
    /// let response = req_resp_service
    ///     .send_request(peer_id, &request, None)
    ///     .await
    ///     .unwrap();
    ///
    /// let data: ExampleResponse = Result::from(response).unwrap();
    /// println!("Received response: {data:?}");
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    #[must_use]
    pub fn req_resp_cbor<Req, Resp>(&self) -> CborReqRespService<Req, Resp>
    where
        Req: Serialize + DeserializeOwned,
        Resp: Serialize + DeserializeOwned,
    {
        CborReqRespService::new(self)
    }

    /// Returns a handle to the scripting service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut scripting_service = connection.scripting();
    ///
    /// let config = ScriptingConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let script_id = scripting_service.deploy_script(config).await.unwrap();
    ///
    /// println!("Deployed script with id on self: {script_id}");
    /// # }
    /// ```
    #[doc(hidden)]
    #[cfg(feature = "scripting")]
    #[must_use]
    pub fn scripting(&self) -> ScriptingService {
        ScriptingService::new(self)
    }
}
