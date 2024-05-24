use drop_stream::DropStream;
use futures::TryStreamExt as _;
use libp2p::PeerId;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use super::{
    script::{self, debug_server::Debug},
    ServerStream, TonicResult,
};
use crate::{debug::Command as DebugCommand, p2p::debug::NeighbourEvent};

impl From<(PeerId, NeighbourEvent)> for script::MeshTopologyEvent {
    fn from((peer_id, event): (PeerId, NeighbourEvent)) -> Self {
        Self {
            peer: script::Peer {
                peer_id: peer_id.to_string(),
            },
            event: script::NeighbourEvent {
                event: Some(match event {
                    NeighbourEvent::Init(peers) => {
                        let peers = peers
                            .into_iter()
                            .map(|peer_id| script::Peer {
                                peer_id: peer_id.to_string(),
                            })
                            .collect();

                        script::neighbour_event::Event::Init(script::Peers { peers })
                    }
                    NeighbourEvent::Discovered(peer_id) => {
                        script::neighbour_event::Event::Discovered(script::Peer {
                            peer_id: peer_id.to_string(),
                        })
                    }
                    NeighbourEvent::Lost(peer_id) => {
                        script::neighbour_event::Event::Lost(script::Peer {
                            peer_id: peer_id.to_string(),
                        })
                    }
                }),
            },
        }
    }
}

pub struct DebugServer {
    command_sender: mpsc::Sender<DebugCommand>,
}

impl DebugServer {
    pub fn new(command_sender: mpsc::Sender<DebugCommand>) -> Self {
        Self { command_sender }
    }
}

#[tonic::async_trait]
impl Debug for DebugServer {
    type SubscribeMeshTopologyStream = ServerStream<script::MeshTopologyEvent>;

    async fn subscribe_mesh_topology(
        &self,
        _request: TonicRequest<script::Empty>,
    ) -> TonicResult<Self::SubscribeMeshTopologyStream> {
        let command_sender = self.command_sender.clone();

        let (sender, receiver) = oneshot::channel();

        command_sender
            .send(DebugCommand::SubscribeNeighbourEvents(sender))
            .await
            .map_err(|_| Status::internal("Failed to send command"))?;

        let receiver = receiver
            .await
            .map_err(|_| Status::internal("Failed to receive response"))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(Into::into)
            .map_err(|e| Status::internal(e.to_string()));

        let drop_stream = DropStream::new(stream, move || {
            tokio::spawn(async move {
                let _ = command_sender
                    .send(DebugCommand::UnsubscribeNeighbourEvents)
                    .await;
            });
        });

        Ok(TonicResponse::new(Box::pin(drop_stream)))
    }
}
