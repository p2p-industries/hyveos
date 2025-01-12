use std::marker::PhantomData;

#[cfg(feature = "serde")]
use derive_where::derive_where;
#[cfg(feature = "serde")]
use futures::future;
use futures::{Stream, StreamExt as _, TryStreamExt as _};
pub use hyveos_core::req_resp::{Request, Response, ResponseError, TopicQuery};
use hyveos_core::{
    grpc::{req_resp_client::ReqRespClient, OptionalTopicQuery, SendRequest, SendResponse},
    req_resp::InboundRequest as CoreInboundRequest,
};
use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A request that was received from a remote peer.
///
/// This struct contains the peer ID of the remote peer, the topic that the request was sent on,
/// and the request data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InboundRequest<T> {
    pub peer_id: PeerId,
    pub topic: Option<String>,
    pub data: T,
}

/// A handle that lets you respond to an inbound request.
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
/// let mut req_resp_service = connection.req_resp();
/// let mut requests = req_resp_service.recv(None).await.unwrap();
///
/// while let Some((request, handle)) = requests.try_next().await.unwrap() {
///     let string = String::from_utf8(request.data).unwrap();
///     println!("Received request from {}: {string}", request.peer_id);
///
///     handle.respond("Hello from the other side!").await.unwrap();
/// }
/// # }
/// ```
#[derive(Debug)]
#[must_use = "requests require a response"]
pub struct InboundRequestHandle<'a> {
    id: u64,
    service: Service,
    phantom: PhantomData<&'a ()>,
}

impl InboundRequestHandle<'_> {
    fn new(id: u64, service: Service) -> Self {
        Self {
            id,
            service,
            phantom: PhantomData,
        }
    }

    /// Responds to this request with a successful response.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
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
    /// let mut req_resp_service = connection.req_resp();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((request, handle)) = requests.try_next().await.unwrap() {
    ///     let string = String::from_utf8(request.data).unwrap();
    ///     println!("Received request from {}: {string}", request.peer_id);
    ///
    ///     handle.respond("Hello from the other side!").await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn respond(mut self, data: impl Into<Vec<u8>>) -> Result<()> {
        self.service
            .respond(self.id, Response::Data(data.into()))
            .await
    }

    /// Responds to this request with an error response.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
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
    /// let mut req_resp_service = connection.req_resp();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((_, handle)) = requests.try_next().await.unwrap() {
    ///     handle.respond_with_error("Something went wrong!").await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn respond_with_error(mut self, error: impl Into<String>) -> Result<()> {
        self.service
            .respond(self.id, Response::Error(error.into().into()))
            .await
    }

    #[doc(hidden)]
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// A typed response that was received from a remote peer.
///
/// The response can either be a successful response with typed data or an error response.
#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub enum TypedResponse<T> {
    Data(T),
    Error(ResponseError),
}

#[cfg(feature = "serde")]
impl<T> From<TypedResponse<T>> for Result<T, ResponseError> {
    fn from(response: TypedResponse<T>) -> Self {
        match response {
            TypedResponse::Data(data) => Ok(data),
            TypedResponse::Error(e) => Err(e),
        }
    }
}

#[cfg(feature = "serde")]
#[doc(hidden)]
#[allow(async_fn_in_trait)]
pub trait TypedService {
    type Resp: Serialize + DeserializeOwned;

    async fn respond(&mut self, id: u64, response: &Self::Resp) -> Result<()>;
    async fn respond_with_error(&mut self, id: u64, error: impl Into<String>) -> Result<()>;
}

/// A handle that lets you respond to an inbound request.
///
/// # Example
///
/// ```no_run
/// use futures::TryStreamExt as _;
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
/// let mut req_resp_service = connection.req_resp_json();
/// let mut requests = req_resp_service.recv(None).await.unwrap();
///
/// while let Some((request, handle)) = requests.try_next().await.unwrap() {
///     let data: ExampleRequest = request.data;
///     println!("Received request from {}: {data:?}", request.peer_id);
///
///     let response = ExampleResponse { message: "Hello from the other side!".to_string() };
///
///     handle.respond(&response).await.unwrap();
/// }
/// # }
/// ```
#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
#[must_use = "requests require a response"]
pub struct TypedInboundRequestHandle<'a, Service, Resp> {
    id: u64,
    service: Service,
    phantom: PhantomData<&'a Resp>,
}

#[cfg(feature = "serde")]
impl<Service, Resp> TypedInboundRequestHandle<'_, Service, Resp>
where
    Service: TypedService<Resp = Resp>,
    Resp: Serialize + DeserializeOwned,
{
    fn new(id: u64, service: Service) -> Self {
        Self {
            id,
            service,
            phantom: PhantomData,
        }
    }

    /// Responds to this request with a successful response.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::TryStreamExt as _;
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
    /// let mut req_resp_service = connection.req_resp_json();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((request, handle)) = requests.try_next().await.unwrap() {
    ///     let data: ExampleRequest = request.data;
    ///     println!("Received request from {}: {data:?}", request.peer_id);
    ///
    ///     let response = ExampleResponse { message: "Hello from the other side!".to_string() };
    ///
    ///     handle.respond(&response).await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn respond(mut self, data: &Resp) -> Result<()> {
        self.service.respond(self.id, data).await
    }

    /// Responds to this request with an error response.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::TryStreamExt as _;
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
    /// let mut req_resp_service = connection.req_resp_json::<ExampleRequest, ExampleResponse>();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((_, handle)) = requests.try_next().await.unwrap() {
    ///     handle.respond_with_error("Something went wrong!").await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn respond_with_error(mut self, error: impl Into<String>) -> Result<()> {
        self.service.respond_with_error(self.id, error).await
    }
}

/// A handle to the request-response service.
///
/// Exposes methods to interact with the request-response service, like for sending and receiving
/// requests, and for sending responses.
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
#[derive(Debug, Clone)]
pub struct Service {
    client: ReqRespClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = ReqRespClient::new(connection.channel.clone());

        Self { client }
    }

    /// Sends a request with an optional topic to a peer and returns the response.
    ///
    /// The peer must be subscribed to the topic (using [`Service::recv`]) in order to receive the
    /// request. If `topic` is `None`, the peer must be subscribed to `None` as well.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
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
    #[tracing::instrument(skip(self, data))]
    pub async fn send_request(
        &mut self,
        peer_id: PeerId,
        data: impl Into<Vec<u8>>,
        topic: Option<String>,
    ) -> Result<Response> {
        let request = Request {
            data: data.into(),
            topic: topic.map(Into::into),
        };

        let request = SendRequest {
            peer: peer_id.into(),
            msg: request.into(),
        };

        self.client
            .send(request)
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }

    /// Subscribes to a topic and returns a stream of tuples of requests sent by peers to this
    /// topic together with an [`InboundRequestHandle`], providing methods to respond to the
    /// request.
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
    /// let mut req_resp_service = connection.req_resp();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((request, handle)) = requests.try_next().await.unwrap() {
    ///     let string = String::from_utf8(request.data).unwrap();
    ///     println!("Received request from {}: {string}", request.peer_id);
    ///
    ///     handle.respond("Hello from the other side!").await.unwrap();
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn recv(
        &mut self,
        query: Option<TopicQuery>,
    ) -> Result<impl Stream<Item = Result<(InboundRequest<Vec<u8>>, InboundRequestHandle<'_>)>>>
    {
        let query = OptionalTopicQuery {
            query: query.map(Into::into),
        };

        self.client
            .recv(query)
            .await
            .map({
                let service = self.clone();
                |response| {
                    response
                        .into_inner()
                        .map_ok(move |req| {
                            let CoreInboundRequest {
                                id,
                                peer_id,
                                req: Request { data, topic },
                            } = req.try_into()?;

                            let request = InboundRequest {
                                peer_id,
                                topic,
                                data,
                            };

                            let handle = InboundRequestHandle::new(id, service.clone());

                            Ok((request, handle))
                        })
                        .map(|res| res?)
                }
            })
            .map_err(Into::into)
    }

    #[doc(hidden)]
    #[tracing::instrument(skip(self))]
    pub async fn respond(&mut self, id: u64, response: Response) -> Result<()> {
        let send_response = SendResponse {
            seq: id,
            response: response.into(),
        };

        self.client
            .respond(send_response)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

/// A handle to the request-response service with JSON-encoded requests and responses.
///
/// Exposes methods to interact with the request-response service, like for sending and receiving
/// requests, and for sending responses.
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
#[derive_where(Debug, Clone)]
pub struct JsonService<Req, Resp> {
    inner: Service,
    _phantom: PhantomData<(Req, Resp)>,
}

#[cfg(feature = "json")]
impl<Req, Resp> JsonService<Req, Resp>
where
    Req: Serialize + DeserializeOwned,
    Resp: Serialize + DeserializeOwned,
{
    pub(crate) fn new(connection: &Connection) -> Self {
        let inner = Service::new(connection);

        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    fn new_from(inner: Service) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Sends a request with an optional topic to a peer and returns the response.
    ///
    /// The peer must be subscribed to the topic (using [`JsonService::recv`]) in order to receive
    /// the request. If `topic` is `None`, the peer must be subscribed to `None` as well.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call, serialization, or deserialization fails.
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
    pub async fn send_request(
        &mut self,
        peer_id: PeerId,
        data: &Req,
        topic: Option<String>,
    ) -> Result<TypedResponse<Resp>> {
        let data = serde_json::to_vec(data)?;

        let response = self.inner.send_request(peer_id, data, topic).await?;

        match response {
            Response::Data(data) => {
                let data = serde_json::from_slice(&data)?;
                Ok(TypedResponse::Data(data))
            }
            Response::Error(error) => Ok(TypedResponse::Error(error)),
        }
    }

    /// Subscribes to a topic and returns a stream of requests sent by peers to this topic, wrapped
    /// into an [`InboundRequestHandle`], providing methods to respond to the request.
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
    /// let mut req_resp_service = connection.req_resp_json();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((request, handle)) = requests.try_next().await.unwrap() {
    ///     let data: ExampleRequest = request.data;
    ///     println!("Received request from {}: {data:?}", request.peer_id);
    ///
    ///     let response = ExampleResponse { message: "Hello from the other side!".to_string() };
    ///
    ///     handle.respond(&response).await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn recv(
        &mut self,
        query: Option<TopicQuery>,
    ) -> Result<
        impl Stream<
            Item = Result<(
                InboundRequest<Req>,
                TypedInboundRequestHandle<'_, Self, Resp>,
            )>,
        >,
    > {
        self.inner.recv(query).await.map(|stream| {
            stream.and_then(|(request, handle)| {
                let InboundRequest {
                    peer_id,
                    topic,
                    data,
                } = request;
                let InboundRequestHandle { id, service, .. } = handle;

                let service = Self::new_from(service);

                future::ready(
                    serde_json::from_slice(&data)
                        .map(|data| {
                            let request = InboundRequest {
                                peer_id,
                                topic,
                                data,
                            };

                            let handle = TypedInboundRequestHandle::new(id, service);

                            (request, handle)
                        })
                        .map_err(Into::into),
                )
            })
        })
    }
}

#[cfg(feature = "json")]
impl<Req, Resp> TypedService for JsonService<Req, Resp>
where
    Req: Serialize + DeserializeOwned,
    Resp: Serialize + DeserializeOwned,
{
    type Resp = Resp;

    async fn respond(&mut self, id: u64, data: &Resp) -> Result<()> {
        let data = serde_json::to_vec(data)?;

        let response = Response::Data(data);

        self.inner.respond(id, response).await
    }

    async fn respond_with_error(&mut self, id: u64, error: impl Into<String>) -> Result<()> {
        let response = Response::Error(error.into().into());

        self.inner.respond(id, response).await
    }
}

/// A handle to the request-response service with CBOR-encoded requests and responses.
///
/// Exposes methods to interact with the request-response service, like for sending and receiving
/// requests, and for sending responses.
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
#[derive_where(Debug, Clone)]
pub struct CborService<Req, Resp> {
    inner: Service,
    _phantom: PhantomData<(Req, Resp)>,
}

#[cfg(feature = "cbor")]
impl<Req, Resp> CborService<Req, Resp>
where
    Req: Serialize + DeserializeOwned,
    Resp: Serialize + DeserializeOwned,
{
    pub(crate) fn new(connection: &Connection) -> Self {
        let inner = Service::new(connection);

        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    fn new_from(inner: Service) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Sends a request with an optional topic to a peer and returns the response.
    ///
    /// The peer must be subscribed to the topic (using [`CborService::recv`]) in order to receive
    /// the request. If `topic` is `None`, the peer must be subscribed to `None` as well.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call, serialization, or deserialization fails.
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
    pub async fn send_request(
        &mut self,
        peer_id: PeerId,
        data: &Req,
        topic: Option<String>,
    ) -> Result<TypedResponse<Resp>> {
        let data = serde_cbor::to_vec(data)?;

        let response = self.inner.send_request(peer_id, data, topic).await?;

        match response {
            Response::Data(data) => {
                let data = serde_cbor::from_slice(&data)?;
                Ok(TypedResponse::Data(data))
            }
            Response::Error(error) => Ok(TypedResponse::Error(error)),
        }
    }

    /// Subscribes to a topic and returns a stream of requests sent by peers to this topic, wrapped
    /// into an [`InboundRequestHandle`], providing methods to respond to the request.
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
    /// let mut req_resp_service = connection.req_resp_cbor();
    /// let mut requests = req_resp_service.recv(None).await.unwrap();
    ///
    /// while let Some((request, handle)) = requests.try_next().await.unwrap() {
    ///     let data: ExampleRequest = request.data;
    ///     println!("Received request from {}: {data:?}", request.peer_id);
    ///
    ///     let response = ExampleResponse { message: "Hello from the other side!".to_string() };
    ///
    ///     handle.respond(&response).await.unwrap();
    /// }
    /// # }
    /// ```
    pub async fn recv(
        &mut self,
        query: Option<TopicQuery>,
    ) -> Result<
        impl Stream<
            Item = Result<(
                InboundRequest<Req>,
                TypedInboundRequestHandle<'_, Self, Resp>,
            )>,
        >,
    > {
        self.inner.recv(query).await.map(|stream| {
            stream.and_then(|(request, handle)| {
                let InboundRequest {
                    peer_id,
                    topic,
                    data,
                } = request;
                let InboundRequestHandle { id, service, .. } = handle;

                let service = Self::new_from(service);

                future::ready(
                    serde_cbor::from_slice(&data)
                        .map(|data| {
                            let request = InboundRequest {
                                peer_id,
                                topic,
                                data,
                            };

                            let handle = TypedInboundRequestHandle::new(id, service);

                            (request, handle)
                        })
                        .map_err(Into::into),
                )
            })
        })
    }
}

#[cfg(feature = "cbor")]
impl<Req, Resp> TypedService for CborService<Req, Resp>
where
    Req: Serialize + DeserializeOwned,
    Resp: Serialize + DeserializeOwned,
{
    type Resp = Resp;

    async fn respond(&mut self, id: u64, data: &Resp) -> Result<()> {
        let data = serde_cbor::to_vec(data)?;

        let response = Response::Data(data);

        self.inner.respond(id, response).await
    }

    async fn respond_with_error(&mut self, id: u64, error: impl Into<String>) -> Result<()> {
        let response = Response::Error(error.into().into());

        self.inner.respond(id, response).await
    }
}
