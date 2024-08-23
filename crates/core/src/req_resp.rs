use std::sync::Arc;

use libp2p_identity::PeerId;
use regex::Regex;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TopicQuery {
    String(Arc<str>),
    #[cfg_attr(feature = "serde", serde(with = "string"))]
    Regex(Regex),
}

impl TopicQuery {
    pub fn matches(&self, topic: impl AsRef<str>) -> bool {
        match self {
            TopicQuery::String(query) => query.as_ref() == topic.as_ref(),
            TopicQuery::Regex(regex) => regex.is_match(topic.as_ref()),
        }
    }
}

impl From<TopicQuery> for grpc::TopicQuery {
    fn from(query: TopicQuery) -> Self {
        let query = match query {
            TopicQuery::String(topic) => grpc::topic_query::Query::Topic(grpc::Topic {
                topic: topic.to_string(),
            }),
            TopicQuery::Regex(regex) => grpc::topic_query::Query::Regex(regex.to_string()),
        };

        Self { query: Some(query) }
    }
}

impl TryFrom<grpc::TopicQuery> for TopicQuery {
    type Error = Error;

    fn try_from(query: grpc::TopicQuery) -> Result<Self> {
        Ok(match query.query.ok_or(Error::MissingQuery)? {
            grpc::topic_query::Query::Topic(topic) => Self::String(topic.topic.into()),
            grpc::topic_query::Query::Regex(regex) => Self::Regex(Regex::new(&regex)?),
        })
    }
}

impl From<String> for TopicQuery {
    fn from(topic: String) -> Self {
        Self::String(topic.into())
    }
}

impl From<&str> for TopicQuery {
    fn from(topic: &str) -> Self {
        Self::String(topic.into())
    }
}

impl From<Arc<str>> for TopicQuery {
    fn from(topic: Arc<str>) -> Self {
        Self::String(topic)
    }
}

impl From<Regex> for TopicQuery {
    fn from(regex: Regex) -> Self {
        Self::Regex(regex)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Request {
    pub data: Vec<u8>,
    pub topic: Option<String>,
}

impl From<Request> for grpc::Message {
    fn from(request: Request) -> Self {
        Self {
            data: request.data,
            topic: grpc::OptionalTopic {
                topic: request.topic.map(|topic| grpc::Topic { topic }),
            },
        }
    }
}

impl From<grpc::Message> for Request {
    fn from(message: grpc::Message) -> Self {
        Self {
            data: message.data,
            topic: message.topic.topic.map(|topic| topic.topic),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InboundRequest {
    pub id: u64,
    pub peer_id: PeerId,
    pub req: Request,
}

impl From<InboundRequest> for grpc::RecvRequest {
    fn from(request: InboundRequest) -> Self {
        Self {
            peer: request.peer_id.into(),
            msg: request.req.into(),
            seq: request.id,
        }
    }
}

impl TryFrom<grpc::RecvRequest> for InboundRequest {
    type Error = Error;

    fn try_from(request: grpc::RecvRequest) -> Result<Self> {
        Ok(Self {
            id: request.seq,
            peer_id: request.peer.try_into()?,
            req: request.msg.into(),
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ResponseError {
    #[error("Request timed out")]
    Timeout,
    #[error(
        "Peer is not subscribed to {}",
        .0.as_deref().map_or("the empty topic".to_string(), |topic| format!("topic '{topic:?}'"))
    )]
    TopicNotSubscribed(Option<String>),
    #[error("Script error: {0}")]
    Script(String),
}

impl From<String> for ResponseError {
    fn from(e: String) -> Self {
        ResponseError::Script(e)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Response {
    Data(Vec<u8>),
    Error(ResponseError),
}

impl From<Response> for grpc::Response {
    fn from(response: Response) -> Self {
        Self {
            response: Some(match response {
                Response::Data(data) => grpc::response::Response::Data(data),
                Response::Error(e) => grpc::response::Response::Error(e.to_string()),
            }),
        }
    }
}

impl From<Response> for Result<Vec<u8>, ResponseError> {
    fn from(response: Response) -> Self {
        match response {
            Response::Data(data) => Ok(data),
            Response::Error(e) => Err(e),
        }
    }
}

impl TryFrom<grpc::Response> for Response {
    type Error = Error;

    fn try_from(response: grpc::Response) -> Result<Self> {
        Ok(match response.response.ok_or(Error::MissingResponse)? {
            grpc::response::Response::Data(data) => Self::Data(data),
            grpc::response::Response::Error(e) => Self::Error(e.into()),
        })
    }
}

#[cfg(feature = "serde")]
mod string {
    use std::{fmt::Display, str::FromStr};

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
