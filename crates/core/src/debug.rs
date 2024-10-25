use libp2p_identity::PeerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    discovery::NeighbourEvent,
    error::{Error, Result},
    gossipsub, grpc,
    req_resp::{Request, Response},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeshTopologyEvent {
    pub peer_id: PeerId,
    pub event: NeighbourEvent,
}

impl From<MeshTopologyEvent> for grpc::MeshTopologyEvent {
    fn from(event: MeshTopologyEvent) -> Self {
        Self {
            peer: event.peer_id.into(),
            event: event.event.into(),
        }
    }
}

impl TryFrom<grpc::MeshTopologyEvent> for MeshTopologyEvent {
    type Error = Error;

    fn try_from(event: grpc::MeshTopologyEvent) -> Result<Self> {
        Ok(Self {
            peer_id: event.peer.try_into()?,
            event: event.event.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RequestDebugEvent {
    pub id: Ulid,
    pub receiver: PeerId,
    pub msg: Request,
}

impl From<RequestDebugEvent> for grpc::RequestDebugEvent {
    fn from(event: RequestDebugEvent) -> Self {
        Self {
            id: event.id.into(),
            receiver: event.receiver.into(),
            msg: event.msg.into(),
        }
    }
}

impl TryFrom<grpc::RequestDebugEvent> for RequestDebugEvent {
    type Error = Error;

    fn try_from(event: grpc::RequestDebugEvent) -> Result<Self> {
        Ok(Self {
            id: event.id.try_into()?,
            receiver: event.receiver.try_into()?,
            msg: event.msg.into(),
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ResponseDebugEvent {
    pub req_id: Ulid,
    pub response: Response,
}

impl From<ResponseDebugEvent> for grpc::ResponseDebugEvent {
    fn from(event: ResponseDebugEvent) -> Self {
        Self {
            req_id: event.req_id.into(),
            response: event.response.into(),
        }
    }
}

impl TryFrom<grpc::ResponseDebugEvent> for ResponseDebugEvent {
    type Error = Error;

    fn try_from(event: grpc::ResponseDebugEvent) -> Result<Self> {
        Ok(Self {
            req_id: event.req_id.try_into()?,
            response: event.response.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MessageDebugEventType {
    Request(RequestDebugEvent),
    Response(ResponseDebugEvent),
    GossipSub(gossipsub::Message),
}

impl From<MessageDebugEventType> for grpc::message_debug_event::Event {
    fn from(event: MessageDebugEventType) -> Self {
        match event {
            MessageDebugEventType::Request(req) => Self::Req(req.into()),
            MessageDebugEventType::Response(res) => Self::Res(res.into()),
            MessageDebugEventType::GossipSub(msg) => Self::Gos(msg.into()),
        }
    }
}

impl TryFrom<grpc::message_debug_event::Event> for MessageDebugEventType {
    type Error = Error;

    fn try_from(event: grpc::message_debug_event::Event) -> Result<Self> {
        Ok(match event {
            grpc::message_debug_event::Event::Req(req) => Self::Request(req.try_into()?),
            grpc::message_debug_event::Event::Res(res) => Self::Response(res.try_into()?),
            grpc::message_debug_event::Event::Gos(msg) => Self::GossipSub(msg.into()),
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageDebugEvent {
    pub sender: PeerId,
    pub event: MessageDebugEventType,
}

impl From<MessageDebugEvent> for grpc::MessageDebugEvent {
    fn from(event: MessageDebugEvent) -> Self {
        Self {
            sender: event.sender.into(),
            event: Some(event.event.into()),
        }
    }
}

impl TryFrom<grpc::MessageDebugEvent> for MessageDebugEvent {
    type Error = Error;

    fn try_from(event: grpc::MessageDebugEvent) -> Result<Self> {
        Ok(Self {
            sender: event.sender.try_into()?,
            event: event.event.ok_or(Error::MissingEvent)?.try_into()?,
        })
    }
}
