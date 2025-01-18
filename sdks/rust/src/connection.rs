use std::{env, path::PathBuf, sync::Arc, time::Duration};

use hyper_util::rt::TokioIo;
use hyveos_core::{
    grpc::{control_client::ControlClient, Empty},
    BRIDGE_SOCKET_ENV_VAR,
};
use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::UnixStream, sync::Mutex, task::JoinHandle};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

#[cfg(feature = "app-management")]
use crate::services::AppsService;
#[cfg(feature = "cbor")]
use crate::services::CborReqRespService;
#[cfg(feature = "json")]
use crate::services::JsonReqRespService;
use crate::{
    error::{Error, Result},
    services::{
        DbService, DebugService, DhtService, DiscoveryService, FileTransferService,
        GossipSubService, NeighboursService, ReqRespService,
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

#[derive(Debug, Clone)]
pub struct DefaultConnection;

/// A connection to the hyveOS runtime through the hyveOS application bridge.
///
/// The Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
/// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
///
/// This is the standard connection type when used in a hyveOS app.
#[derive(Debug, Clone)]
pub struct ApplicationConnection {
    heartbeat_interval: Duration,
}

impl Default for ApplicationConnection {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(10),
        }
    }
}

impl internal::ConnectionType for ApplicationConnection {
    async fn connect(self) -> Result<Connection> {
        let channel = Endpoint::try_from("http://[::]:50051")? // dummy URI
            .connect_with_connector(service_fn(|_: Uri| async {
                let path = env::var(BRIDGE_SOCKET_ENV_VAR)
                    .map_err(|e| Error::EnvVarMissing(BRIDGE_SOCKET_ENV_VAR, e))?;

                UnixStream::connect(path)
                    .await
                    .map_err(Error::from)
                    .map(TokioIo::new)
            }))
            .await?;

        let control = Arc::new(Mutex::new(ControlClient::new(channel.clone())));

        let heartbeat_task = tokio::spawn({
            let control = control.clone();
            async move {
                let mut retries = 0;
                loop {
                    if let Err(e) = control.lock().await.heartbeat(Empty {}).await {
                        tracing::error!(retries, "Failed to send heartbeat: {e}");
                        if retries >= 3 {
                            break;
                        }

                        retries += 1;
                        continue;
                    }

                    retries = 0;

                    tokio::time::sleep(self.heartbeat_interval).await;
                }
            }
        });

        Ok(Connection {
            channel,
            #[cfg(feature = "network")]
            reqwest_client_and_url: None,
            shared_dir_path: None,
            control,
            heartbeat_task: Some(heartbeat_task),
        })
    }
}

/// A connection to the hyveOS runtime through a custom Unix domain socket.
#[derive(Debug, Clone)]
pub struct CustomConnection {
    socket_path: PathBuf,
    shared_dir_path: PathBuf,
}

impl internal::ConnectionType for CustomConnection {
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
            channel: channel.clone(),
            #[cfg(feature = "network")]
            reqwest_client_and_url: None,
            shared_dir_path: Some(Arc::new(self.shared_dir_path)),
            control: Arc::new(Mutex::new(ControlClient::new(channel))),
            heartbeat_task: None,
        })
    }
}

/// A connection over the network to a hyveOS runtime listening at a given URI.
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

        #[cfg_attr(
            not(any(target_os = "android", target_os = "fuchsia", target_os = "linux")),
            expect(unused_mut)
        )]
        let mut client_builder = reqwest::Client::builder();

        #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
        if let Some(if_name) = if_name {
            client_builder = client_builder.interface(&if_name);
        }
        #[cfg(not(any(target_os = "android", target_os = "fuchsia", target_os = "linux")))]
        assert!(
            if_name.is_none(),
            "Interface name in URI is only supported on Android, Fuchsia, and Linux"
        );

        let client = client_builder.build()?;

        Ok(Connection {
            channel: channel.clone(),
            reqwest_client_and_url: Some((client, url)),
            shared_dir_path: None,
            control: Arc::new(Mutex::new(ControlClient::new(channel))),
            heartbeat_task: None,
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

/// A builder for configuring a connection to the hyveOS runtime.
#[derive(Debug, Clone)]
pub struct ConnectionBuilder<T> {
    connection_type: T,
}

impl Default for ConnectionBuilder<DefaultConnection> {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionBuilder<DefaultConnection> {
    /// Creates a new builder for configuring a connection to the hyveOS runtime.
    ///
    /// By default, the connection to the hyveOS runtime will be made through the application bridge,
    /// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`Self::custom`] or [`Self::uri`] methods.
    ///
    /// If the connected through the application bridge, the connection will send heartbeat messages
    /// at regular intervals (configurable using [`Self::heartbeat_interval`]) to the hyveOS runtime
    /// to keep the connection alive.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_type: DefaultConnection,
        }
    }

    /// Sets the interval at which the connection should send heartbeat messages to the hyveOS runtime.
    ///
    /// The default interval is 10 seconds.
    /// By default, the runtime will close the connection if no heartbeat messages are received within 20 seconds.
    #[must_use]
    pub fn heartbeat_interval(
        self,
        interval: Duration,
    ) -> ConnectionBuilder<ApplicationConnection> {
        ConnectionBuilder {
            connection_type: ApplicationConnection {
                heartbeat_interval: interval,
            },
        }
    }

    /// Specifies a custom Unix domain socket to connect to.
    ///
    /// The socket path should point to a Unix domain socket that the hyveOS runtime is listening on.
    /// The shared directory path should point to the shared directory that the hyveOS runtime is using.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::builder()
    ///     .custom("/path/to/hyveos.sock", "/path/to/shared/dir")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn custom(
        self,
        socket_path: impl Into<PathBuf>,
        shared_dir_path: impl Into<PathBuf>,
    ) -> ConnectionBuilder<CustomConnection> {
        ConnectionBuilder {
            connection_type: CustomConnection {
                socket_path: socket_path.into(),
                shared_dir_path: shared_dir_path.into(),
            },
        }
    }

    /// Specifies a URI to connect to over the network.
    ///
    /// The URI should be in the format `http://<host>:<port>`.
    /// A hyveOS runtime should be listening at the given address.
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
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[cfg(feature = "network")]
    #[must_use]
    pub fn uri(self, uri: Uri) -> ConnectionBuilder<UriConnection> {
        ConnectionBuilder {
            connection_type: UriConnection { uri },
        }
    }

    /// Establishes a connection to the hyveOS runtime.
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
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub async fn connect(self) -> Result<Connection> {
        internal::ConnectionType::connect(ApplicationConnection::default()).await
    }
}

impl ConnectionBuilder<ApplicationConnection> {
    /// Sets the interval at which the connection should send heartbeat messages to the hyveOS runtime.
    ///
    /// The default interval is 10 seconds.
    /// By default, the runtime will close the connection if no heartbeat messages are received within 20 seconds.
    #[must_use]
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.connection_type.heartbeat_interval = interval;
        self
    }
}

impl<T: ConnectionType> ConnectionBuilder<T> {
    /// Establishes a connection to the hyveOS runtime.
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
    ///     .custom("/path/to/hyveos.sock", "/path/to/shared/dir")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub async fn connect(self) -> Result<Connection> {
        self.connection_type.connect().await
    }
}

/// A connection to the hyveOS runtime.
///
/// This struct provides access to the various services provided by hyveOS.
///
/// By default, the connection to the hyveOS runtime will be made through the application bridge,
/// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
/// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
/// If another connection type is desired, use the [`Self::builder`] function to get a
/// [`ConnectionBuilder`] and use the [`ConnectionBuilder::custom`] or
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
/// let peer_id = connection.get_id().await.unwrap();
///
/// println!("My peer id: {peer_id}");
/// # }
/// ```
pub struct Connection {
    pub(crate) channel: Channel,
    #[cfg(feature = "network")]
    pub(crate) reqwest_client_and_url: Option<(reqwest::Client, reqwest::Url)>,
    pub(crate) shared_dir_path: Option<Arc<PathBuf>>,
    control: Arc<Mutex<ControlClient<Channel>>>,
    heartbeat_task: Option<JoinHandle<()>>,
}

impl Connection {
    /// Establishes a connection to the hyveOS runtime through the application bridge.
    ///
    /// The Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`Self::builder`] function to get a
    /// [`ConnectionBuilder`] and use the [`ConnectionBuilder::custom`] or
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
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        Connection::builder().connect().await
    }

    /// Creates a new builder for configuring a connection to the hyveOS runtime.
    ///
    /// By default, the connection to the hyveOS runtime will be made through the application bridge,
    /// i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable
    /// ([`hyveos_core::BRIDGE_SOCKET_ENV_VAR`]) will be used to communicate with the runtime.
    /// If another connection type is desired, use the [`ConnectionBuilder::custom`] or
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
    ///     .custom("/path/to/hyveos.sock", "/path/to/shared/dir")
    ///     .connect()
    ///     .await
    ///     .unwrap();
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> ConnectionBuilder<DefaultConnection> {
        ConnectionBuilder::new()
    }

    /// Returns a handle to the application management service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::{Connection, services::AppConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut apps_service = connection.apps();
    ///
    /// let config = AppConfig::new("my-docker-image:latest")
    ///     .local()
    ///     .expose_port(8080);
    /// let app_id = apps_service.deploy(config).await.unwrap();
    ///
    /// println!("Deployed app with id {app_id}");
    /// # }
    /// ```
    #[doc(hidden)]
    #[cfg(feature = "app-management")]
    #[must_use]
    pub fn apps(&self) -> AppsService {
        AppsService::new(self)
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
    /// discovery_service.provide("topic", "key").await.unwrap();
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
    /// let cid = file_transfer_service.publish(&file_path).await.unwrap();
    ///
    /// println!("Content ID: {cid:?}");
    /// # }
    /// ```
    #[must_use]
    pub fn file_transfer(&self) -> FileTransferService {
        FileTransferService::new(self)
    }

    /// Returns a handle to the distributed key-value store service.
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
    #[must_use]
    pub fn kv(&self) -> DhtService {
        DhtService::new(self)
    }

    /// Returns a handle to the local key-value store service.
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
    #[must_use]
    pub fn local_kv(&self) -> DbService {
        DbService::new(self)
    }

    /// Returns a handle to the neighbours service.
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
    #[must_use]
    pub fn neighbours(&self) -> NeighboursService {
        NeighboursService::new(self)
    }

    /// Returns a handle to the pub-sub service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut pub_sub_service = connection.pub_sub();
    /// let id = pub_sub_service.publish("topic", "Hello, world!").await.unwrap();
    ///
    /// println!("Published message with id: {id}");
    /// # }
    /// ```
    #[must_use]
    pub fn pub_sub(&self) -> GossipSubService {
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
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service
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
    /// let data = Vec::try_from(response).unwrap();
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
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service
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

    /// Returns a handle to the request-response service with CBOR-encoded requests and responses.
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
    /// let mut discovery_service = connection.discovery();
    /// let peer_id = discovery_service
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

    /// Returns the peer ID of the local runtime.
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
    /// let peer_id = connection.get_id().await.unwrap();
    ///
    /// println!("My peer id: {peer_id}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_id(&self) -> Result<PeerId> {
        self.control
            .lock()
            .await
            .get_id(Empty {})
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(heartbeat_task) = self.heartbeat_task.take() {
            heartbeat_task.abort();
        }
    }
}
