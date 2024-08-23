use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageId(pub Vec<u8>);

impl From<MessageId> for grpc::GossipSubMessageId {
    fn from(id: MessageId) -> Self {
        Self { id: id.0 }
    }
}

impl From<grpc::GossipSubMessageId> for MessageId {
    fn from(id: grpc::GossipSubMessageId) -> Self {
        Self(id.id)
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", hex_fmt::HexFmt(&self.0))
    }
}

impl Debug for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("MessageId")
            .field(&hex_fmt::HexFmt(&self.0))
            .finish()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Message {
    pub data: Vec<u8>,
    pub topic: String,
}

impl From<Message> for grpc::GossipSubMessage {
    fn from(message: Message) -> Self {
        Self {
            data: message.data,
            topic: grpc::Topic {
                topic: message.topic,
            },
        }
    }
}

impl From<grpc::GossipSubMessage> for Message {
    fn from(message: grpc::GossipSubMessage) -> Self {
        Self {
            data: message.data,
            topic: message.topic.topic,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReceivedMessage {
    pub propagation_source: PeerId,
    pub source: Option<PeerId>,
    pub message_id: MessageId,
    pub message: Message,
}

impl From<ReceivedMessage> for grpc::GossipSubRecvMessage {
    fn from(message: ReceivedMessage) -> Self {
        Self {
            propagation_source: message.propagation_source.into(),
            source: message.source.map(Into::into),
            msg: message.message.into(),
            msg_id: message.message_id.into(),
        }
    }
}

impl TryFrom<grpc::GossipSubRecvMessage> for ReceivedMessage {
    type Error = Error;

    fn try_from(message: grpc::GossipSubRecvMessage) -> Result<Self> {
        Ok(Self {
            propagation_source: message.propagation_source.try_into()?,
            source: message.source.map(TryInto::try_into).transpose()?,
            message_id: message.msg_id.into(),
            message: message.msg.into(),
        })
    }
}
