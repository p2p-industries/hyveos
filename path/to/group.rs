pub enum GroupCommand {
    CreateGroup {
        name: String,
        leader_id: PeerId,
        sender: oneshot::Sender<GroupId>,
    },
    InviteMember {
        group_id: GroupId,
        invitee_id: PeerId,
        sender: oneshot::Sender<InvitationId>,
    },
    RespondToInvitation {
        invitation_id: InvitationId,
        accept: bool,
        // sender: oneshot::Sender<Result<GroupId, InvitationError>>,
    },
    LeaveGroup {
        group_id: GroupId,
        member_id: PeerId,
        sender: oneshot::Sender<Result<(), GroupError>>,
    },
    GetGroupInfo {
        group_id: GroupId,
        sender: oneshot::Sender<Option<GroupInfo>>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GroupId(Ulid);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InvitationId(Ulid);

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub leader_id: PeerId,
    pub members: HashSet<PeerId>,
    pub pending_invites: HashSet<PeerId>,
}
