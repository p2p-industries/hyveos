use crate::req_resp::{Request, Response};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ulid::Ulid;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupId(pub Ulid);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitationId(pub Ulid);

#[derive(Debug, Serialize, Deserialize)]
pub enum GroupRequest {
    Invitation {
        group_id: GroupId,
        group_name: String,
        from_peer: PeerId,
    },
    InvitationResponse {
        invitation_id: InvitationId,
        accepted: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GroupResponse {
    InvitationAck { invitation_id: InvitationId },
    InvitationResponseAck { success: bool },
}

impl TryFrom<GroupRequest> for Request {
    type Error = serde_cbor::Error;

    fn try_from(req: GroupRequest) -> Result<Self, Self::Error> {
        Ok(Request {
            topic: Some("group".into()),
            data: serde_cbor::to_vec(&req)?,
        })
    }
}

impl TryFrom<GroupResponse> for Response {
    type Error = serde_cbor::Error;

    fn try_from(resp: GroupResponse) -> Result<Self, Self::Error> {
        Ok(Response::Data(serde_cbor::to_vec(&resp)?))
    }
}

// Core group types
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub leader_id: PeerId,
    pub members: HashSet<PeerId>,
    pub pending_invites: HashSet<PeerId>,
}
