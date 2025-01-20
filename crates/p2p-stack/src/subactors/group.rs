use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use libp2p::PeerId;
use tokio::sync::{broadcast, mpsc, oneshot};
use ulid::Ulid;
use serde::{Serialize, Deserialize};
use hyveos_core::group::{GroupRequest, GroupResponse, GroupId, InvitationId, GroupInfo};
use serde_cbor;

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    subactors::req_resp::{self, InboundRequest, Response},
};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct GroupId(Ulid);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvitationId(Ulid);

// TODO: leave out the pub struct Actor in here for now, and think about where the others use it 
// and whether it is necessary

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
    }
}

#[derive(Debug)]
pub struct GroupState {
    // Core group data
    name: String,
    members: HashSet<PeerId>,
    pending_invites: HashSet<PeerId>,
    current_leader: Option<PeerId>, // Updated by RAFT consensus
}

#[derive(Debug)]
pub struct RaftState {
    // RAFT protocol state
    current_term: u64,
    voted_for: Option<PeerId>,
    log: Vec<RaftLogEntry>,
    commit_index: u64,
    last_applied: u64,
    current_role: RaftRole,
    election_timeout: Duration,
    last_heartbeat: Instant,
    // votes_received: HashSet<PeerId>,
}

#[derive(Debug, Serialize, Deserialize)] 
pub enum GroupResponse {
    InvitationAck {
        invitation_id: InvitationId,
    },
    InvitationResponseAck {
        success: bool,
    }
}

impl From<GroupResponse> for Response { 
    fn from(response: GroupResponse) -> Self {
        // Serialize GroupResponse into bytes for Response::Data
        let bytes = serde_cbor::to_vec(&response).expect("GroupResponse serialization failed");
        Response::Data(bytes)
    }
}

pub enum Command {
    CreateGroup {
        group_id: GroupId,
        name: String,
        sender: oneshot::Sender<Result<Group, GroupError>>,
    },

    // Outbound invitation handling
    SendInvitation {
        group_id: GroupId,
        peer_id: PeerId,
        sender: oneshot::Sender<Result<Group, GroupError>>,
    },

    // Inbound invitation handling (received via req_resp)
    HandleInboundInvitation {
        request: InboundRequest<GroupRequest>,
        response_channel: req_resp::ResponseChannel<GroupResponse>,
    },

    // Response handling
    HandleInvitationResponse {
        from_peer: PeerId,
        response: GroupResponse,
    },

    // Subscribe to incoming invitations
    SubscribeInvites {
        sender: oneshot::Sender<(SubscriptionId, mpsc::Receiver<GroupInvite>)>,
    },

    // RAFT related
    HandleRaftMessage {
        from_peer: PeerId,
        message: RaftMessage,
    },
    // Invitation handling
    HandleInvitation {
        from_peer: PeerId,
        group_id: GroupId,
        sender: oneshot::Sender<Result<(), GroupError>>,
    },
    RespondToInvitation {

    },
    GroupInvite {
        peer_id: PeerId,
        group_id: GroupId,
        group: Group,
        req: req_resp::Request, 
        sender: oneshot::Sender<Response>,
    },
    Subscribe {
        group: Option<TopicQuery>,
        sender: oneshot::Sender<(SubscriptionId, mpsc::Receiver<InboundRequest>)>,
    },
    Unsubscribe(SubscriptionId), // can a node just unsubscribe? -- no. 
    AcceptInvite {
        id: u64,
        response: Response,
    },
    DeclineInvite {
        id: u64,
        response: Response,
    },
    DebugSubscribe(oneshot::Sender<broadcast::Receiver<MessageDebugEventType>>),
}


impl SubActor for Actor {
    type SubCommand = Command;
    type Event = <req_resp::Behaviour as NetworkBehaviour>::ToSwarm;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id: Some(peer_id),
            ..Default::default()
        }
    }


    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::CreateGroup { 
                name, 
                sender 
            } => {
                let group_id = GroupId(Ulid::new());

                // Initialize group state
                let mut group_state = GroupState {
                    name: name.clone(),
                    members: HashSet::from([self.peer_id.unwrap()]),
                    pending_invites: HashSet::new(),
                    current_leader: Some(self.peer_id.unwrap()),
                };

                // Initialize RAFT state
                let raft_state = RaftState {
                    current_term: 0,
                    voted_for: None,
                    log: Vec::new(),
                    commit_index: 0,
                    last_applied: 0,
                    current_role: RaftRole::Leader, // Creator starts as leader
                    election_timeout: Duration::from_secs(5),
                    last_heartbeat: Instant::now(),
                };

                self.groups.insert(group_id.clone(), group_state);
                self.raft_states.insert(group_id.clone(), raft_state);

                // Create Group object for SDK
                let group = Group {
                    id: group_id.clone(),
                    name,
                    leader_id: self.peer_id.unwrap(),
                };

                let _ = sender.send(Ok(group));
                Ok(())
            },

            Command::SendInvitation { group_id, peer_id, sender } => {
                let group = self.groups.get(&group_id).ok_or(GroupError::GroupNotFound)?;
                let invitation_id = InvitationId(Ulid::new());

                let request = GroupRequest::Invitaton {
                    group_id: group_id.clone(),
                    group_name: group.name.clone(),
                    from_peer: self.peer_id.unwrap(),
                };

                // use req_resp to
                let outbound_request = behaviour.req_resp.send_request(
                    &peer_id,
                    request,
                );
                self.response_senders.insert(outbound_request, sender);
                Ok(())
            },

            Command::HandleInboundInvitation {
                request, response_channel
            } => {
                if let GroupRequest::Invitation { group_id, group_name, from_peer } = request.req {
                    self.request_channel.insert(request.id, response_channel);

                    // Notify subscribers about new invitation
                    for subscriber in self.invite_subscribers.values() {
                        let _ = subscriber.try_send(GroupInvite {
                            id: request.id,
                            group_id: group_id.clone(),
                            from_peer,
                            group_name: group_name.clone(),
                        });
                    }
                }
                Ok(())
            },

            Command::RespondToInvitation {
                invitation_id,
                accept
            } => {
                if let Some(response_channel) = self.request_channel.remove(&invitation_id) {
                    let group_response = GroupResponse::InvitationResponseAck {
                        success: accept,
                    };

                    let response: Response = group_response.into();

                    behaviour.req_resp.send_response(
                        response_channel,
                        response,
                    );
                }
                Ok(())
            },
        }
    }



    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::Message { peer, message } => match message {
                Message::Request {
                    request_id,
                    request: Request { debug_id, req },
                    channel,
                } => {
                    // This is safe because InboundRequestId is a newtype around u64
                    let id = unsafe { std::mem::transmute::<InboundRequestId, u64>(request_id) };
                    let topic = req.topic.clone();

                    tracing::debug!("Received request with id {id} from peer {peer}");

                    let request = InboundRequest {
                        id,
                        peer_id: peer,
                        req,
                    };

                    let mut sent_to_subscriber = false;
                    for (query, sender) in self.request_subscriptions.values() {
                        let is_match = match (query.as_ref(), topic.as_ref()) {
                            (Some(query), Some(topic)) => query.matches(topic),
                            (None, None) => true,
                            _ => false,
                        };

                        if is_match && sender.try_send(request.clone()).is_ok() {
                            sent_to_subscriber = true;
                        }
                    }

                    if sent_to_subscriber {
                        self.response_channels.insert(id, Ok((debug_id, channel)));
                    } else {
                        let response = Response::Error(ResponseError::TopicNotSubscribed(topic));

                        self.send_debug_event(|| {
                            MessageDebugEventType::Response(ResponseDebugEvent {
                                req_id: debug_id,
                                response: response.clone(),
                            })
                        });

                        let _ = behaviour.req_resp.send_response(channel, response);
                    }
                }
                Message::Response {
                    request_id, response
                } => match message {
                    if let Response::Data(bytes) = response {
                        if let Ok(group_response) = serde_cbor::from_slice::<GroupResponse>(&bytes) {
                            // TODO: handle group response
                        }
                    }
                    tracing::debug!("Received response for request with id {request_id}");

                    if let Some(sender) = self.response_senders.remove(&request_id) {
                        sender.send(response).unwrap();
                    }
                }
            },
            e => {
                tracing::debug!("Unhandled event: {e:?}");
            }
        }
        Ok(())
    }
}
