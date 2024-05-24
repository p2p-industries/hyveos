use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use super::{
    script::{self, discovery_server::Discovery},
    ServerStream, TonicResult,
};
use crate::p2p::Client;

#[cfg(feature = "batman")]
use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
};
#[cfg(feature = "batman")]
use tokio_stream::wrappers::BroadcastStream;

#[cfg(feature = "batman")]
use crate::p2p::neighbours::Event;

pub struct DiscoveryServer {
    client: Client,
}

impl DiscoveryServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl Discovery for DiscoveryServer {
    type SubscribeEventsStream = ServerStream<script::NeighbourEvent>;

    #[cfg(feature = "batman")]
    async fn subscribe_events(
        &self,
        _request: TonicRequest<script::Empty>,
    ) -> TonicResult<Self::SubscribeEventsStream> {
        let neighbours = self.client.neighbours();

        let resolved = neighbours
            .get_resolved()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let peers = script::Peers {
            peers: resolved
                .keys()
                .copied()
                .map(|id| script::Peer {
                    peer_id: id.to_string(),
                })
                .collect::<Vec<_>>(),
        };

        let init = script::NeighbourEvent {
            event: Some(script::neighbour_event::Event::Init(peers)),
        };

        let sub = neighbours
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let sub_stream = BroadcastStream::new(sub)
            .map_ok(|event| match event.as_ref() {
                Event::ResolvedNeighbour(neighbour) => script::NeighbourEvent {
                    event: Some(script::neighbour_event::Event::Discovered(script::Peer {
                        peer_id: neighbour.peer_id.to_string(),
                    })),
                },
                Event::LostNeighbour(neighbour) => script::NeighbourEvent {
                    event: Some(script::neighbour_event::Event::Lost(script::Peer {
                        peer_id: neighbour.peer_id.to_string(),
                    })),
                },
            })
            .map_err(|e| Status::internal(e.to_string()));

        Ok(TonicResponse::new(Box::pin(
            stream::once(future::ready(Ok(init))).chain(sub_stream),
        )))
    }

    #[cfg(not(feature = "batman"))]
    async fn subscribe_events(
        &self,
        _request: TonicRequest<script::Empty>,
    ) -> TonicResult<Self::SubscribeEventsStream> {
        return Err(Status::unavailable("batman feature is not enabled"));
    }

    async fn get_own_id(&self, _request: TonicRequest<script::Empty>) -> TonicResult<script::Peer> {
        let id = self.client.peer_id();

        Ok(TonicResponse::new(script::Peer {
            peer_id: id.to_string(),
        }))
    }
}
