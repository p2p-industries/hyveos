#[cfg(feature = "serde")]
use futures::future;
use futures::{Stream, StreamExt as _, TryStreamExt as _};
pub use hyveos_core::pub_sub::Message;
use hyveos_core::{
    grpc::{pub_sub_client::PubSubClient, PubSubMessage, Topic},
    pub_sub::{MessageId, ReceivedMessage},
};
#[cfg(feature = "serde")]
use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tonic::transport::Channel;

use crate::{connection::Connection, error::Result};

/// A typed message received from the pub-sub service.
///
/// This struct contains the typed data of a message received from the pub-sub service, along
/// with metadata like the source of the message and the topic it was published to.
#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub struct TypedMessage<T> {
    pub propagation_source: PeerId,
    pub source: Option<PeerId>,
    pub message_id: MessageId,
    pub topic: String,
    pub data: T,
}

/// A handle to the pub-sub service.
///
/// Exposes methods to interact with the pub-sub service,
/// like for subscribing to topics and publishing messages.
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
#[derive(Debug, Clone)]
pub struct Service {
    client: PubSubClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = PubSubClient::new(connection.channel.clone());

        Self { client }
    }

    /// Subscribes to a topic and returns a stream of messages published to that topic.
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let mut messages = pub_sub_service.subscribe("topic").await.unwrap();
    ///
    /// while let Some(message) = messages.try_next().await.unwrap() {
    ///     let string = String::from_utf8(message.message.data).unwrap();
    ///
    ///     if let Some(source) = message.source {
    ///         let direct_source = message.propagation_source;
    ///         println!("Received message from {source} via {direct_source}: {string}");
    ///     } else {
    ///         println!("Received message from unknown source: {string}");
    ///     }
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self, topic), fields(topic))]
    pub async fn subscribe(
        &mut self,
        topic: impl Into<String>,
    ) -> Result<impl Stream<Item = Result<ReceivedMessage>>> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let topic = Topic { topic };

        self.client
            .subscribe(topic)
            .await
            .map(|response| {
                response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Subscribes to a topic and returns a stream of JSON-encoded messages published to that
    /// topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails. The stream emits errors that occur in the runtime
    /// while processing the providers, as well as data conversion and deserialization errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::TryStreamExt as _;
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let mut messages = pub_sub_service.subscribe_json("topic").await.unwrap();
    ///
    /// while let Some(message) = messages.try_next().await.unwrap() {
    ///     let data: Example = message.data;
    ///
    ///     if let Some(source) = message.source {
    ///         let direct_source = message.propagation_source;
    ///         println!("Received message from {source} via {direct_source}: {data:?}");
    ///     } else {
    ///         println!("Received message from unknown source: {data:?}");
    ///     }
    /// }
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn subscribe_json<T: DeserializeOwned>(
        &mut self,
        topic: impl Into<String>,
    ) -> Result<impl Stream<Item = Result<TypedMessage<T>>>> {
        self.subscribe(topic).await.map(|stream| {
            stream.and_then(|msg| {
                let ReceivedMessage {
                    propagation_source,
                    source,
                    message_id,
                    message: Message { topic, data },
                } = msg;

                future::ready(
                    serde_json::from_slice(&data)
                        .map(|data| TypedMessage {
                            propagation_source,
                            source,
                            message_id,
                            topic,
                            data,
                        })
                        .map_err(Into::into),
                )
            })
        })
    }

    /// Subscribes to a topic and returns a stream of CBOR-encoded messages published to that
    /// topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails. The stream emits errors that occur in the runtime
    /// while processing the providers, as well as data conversion and deserialization errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::TryStreamExt as _;
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let mut messages = pub_sub_service.subscribe_cbor("topic").await.unwrap();
    ///
    /// while let Some(message) = messages.try_next().await.unwrap() {
    ///     let data: Example = message.data;
    ///
    ///     if let Some(source) = message.source {
    ///         let direct_source = message.propagation_source;
    ///         println!("Received message from {source} via {direct_source}: {data:?}");
    ///     } else {
    ///         println!("Received message from unknown source: {data:?}");
    ///     }
    /// }
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn subscribe_cbor<T: DeserializeOwned>(
        &mut self,
        topic: impl Into<String>,
    ) -> Result<impl Stream<Item = Result<TypedMessage<T>>>> {
        self.subscribe(topic).await.map(|stream| {
            stream.and_then(|msg| {
                let ReceivedMessage {
                    propagation_source,
                    source,
                    message_id,
                    message: Message { topic, data },
                } = msg;

                future::ready(
                    serde_cbor::from_slice(&data)
                        .map(|data| TypedMessage {
                            propagation_source,
                            source,
                            message_id,
                            topic,
                            data,
                        })
                        .map_err(Into::into),
                )
            })
        })
    }

    /// Publishes a message to a topic.
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let id = pub_sub_service.publish("topic", "Hello, world!").await.unwrap();
    ///
    /// println!("Published message with id: {id}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self, topic, data), fields(topic))]
    pub async fn publish(
        &mut self,
        topic: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) -> Result<MessageId> {
        let topic = topic.into();

        tracing::Span::current().record("topic", &topic);

        let message = Message {
            topic,
            data: data.into(),
        };

        self.client
            .publish(PubSubMessage::from(message))
            .await
            .map(|response| response.into_inner().into())
            .map_err(Into::into)
    }

    /// Publishes a JSON-encoded message to a topic.
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let data = Example { message: "Hello, world!".to_string() };
    /// let id = pub_sub_service.publish_json("topic", &data).await.unwrap();
    ///
    /// println!("Published message with id: {id}");
    /// # }
    /// ```
    #[cfg(feature = "json")]
    pub async fn publish_json(
        &mut self,
        topic: impl Into<String>,
        data: &impl Serialize,
    ) -> Result<MessageId> {
        let data = serde_json::to_vec(&data)?;

        self.publish(topic, data).await
    }

    /// Publishes a CBOR-encoded message to a topic.
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
    /// let mut pub_sub_service = connection.pub_sub();
    /// let data = Example { message: "Hello, world!".to_string() };
    /// let id = pub_sub_service.publish_cbor("topic", &data).await.unwrap();
    ///
    /// println!("Published message with id: {id}");
    /// # }
    /// ```
    #[cfg(feature = "cbor")]
    pub async fn publish_cbor(
        &mut self,
        topic: impl Into<String>,
        data: &impl Serialize,
    ) -> Result<MessageId> {
        let data = serde_cbor::to_vec(&data)?;

        self.publish(topic, data).await
    }
}
