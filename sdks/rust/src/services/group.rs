use futures::future;
use futures::{Stream, StreamExt as _, TryStreamExt as _};
use tokio_stream::wrappers::ReceiverStream;
use libp2p_identity::PeerId;

use crate::{connection::Connection, error::Result};

use hyveos_core::group::{GroupId, GroupInviteEvent, GroupError};
use hyveos_core::grpc::{group_client::GroupClient, Empty};

/// A handle to the group service.
#[derive(Debug, Clone)]
pub struct Service {
    client: GroupClient<Channel>,
}

/// An invite into a group that was received from a peer.
/// Contains the group ID and the peer ID of the peer that sent the invite.
/// The invite can be accepted or rejected.
/// If accepted, the peer will join the group.
/// If rejected, the invite will be removed.
/// It contains the group object as well.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupInvite {
    pub group: Group,
    pub group_id: GroupId,
    pub peer_id: PeerId,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = GroupClient::
            new(connection.channel.clone());

        Self { client }
    }

    // initiate a group creation
    // will create a group with the given name, if successful
    // afterwards, an object Group comes out. On this Group objects, you can call a bunch of methods
    // important: a new Group object comes out of this.
    pub async fn create(&mut self, name: String) -> Result<Group> {
        self.client
            .create(GroupCreateRequest { name })
            .await?;

        Ok(group_id)
    }

    // Returns a stream of GroupInviteEvents.
    pub async fn get_invites() -> Result<impl Stream<Item = GroupInviteEvent>> {
        let (_sub_id, rx) = self.client.subscribe_invites().await?;
        Ok(ReceiverStream::new(rx))
    }

    pub async fn accept(&mut self, invite_id: ulid::Ulid) -> Result<GroupId> {

    }

}
