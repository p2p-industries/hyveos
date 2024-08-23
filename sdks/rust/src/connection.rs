use std::env;

use async_lazy::Lazy;
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
        DebugService, DhtService, DiscoveryService, FileTransferService, GossipSubService,
        ReqRespService,
    },
};

static CONNECTION: Lazy<Result<P2PConnection>> = Lazy::const_new(|| {
    Box::pin(async {
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(|_: Uri| async {
                let path = env::var("P2P_INDUSTRIES_BRIDGE_SOCKET")
                    .map_err(|e| Error::EnvVarMissing("P2P_INDUSTRIES_BRIDGE_SOCKET", e))?;

                UnixStream::connect(path).await.map_err(Error::from)
            }))
            .await?;

        Ok(P2PConnection { channel })
    })
});

/// A connection to the P2P Industries bridge.
///
/// This struct provides access to the various services provided by the bridge.
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
pub struct P2PConnection {
    pub(crate) channel: Channel,
}

impl P2PConnection {
    /// Returns a reference to the P2P Industries bridge connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to the bridge could not be established.
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
    pub async fn get() -> Result<&'static P2PConnection, &'static Error> {
        CONNECTION.force().await.as_ref()
    }

    /// Returns a handle to the debug service.
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
    #[must_use]
    pub fn dht(&self) -> DhtService {
        DhtService::new(self)
    }

    /// Returns a handle to the discovery service.
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
    /// use p2p_industries_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let shared_dir = std::env::var("P2P_INDUSTRIES_SHARED_DIR").unwrap();
    /// let file_path = Path::new(&shared_dir).join("example.txt");
    /// tokio::fs::write(&file_path, "Hello, world!").await.unwrap();
    ///
    /// let connection = P2PConnection::get().await.unwrap();
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
    /// use p2p_industries_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
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
    /// use p2p_industries_sdk::P2PConnection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
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
    /// use p2p_industries_sdk::P2PConnection;
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
    /// let connection = P2PConnection::get().await.unwrap();
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
    /// use p2p_industries_sdk::P2PConnection;
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
    /// let connection = P2PConnection::get().await.unwrap();
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
    /// use p2p_industries_sdk::{P2PConnection, services::ScriptingConfig};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = P2PConnection::get().await.unwrap();
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
