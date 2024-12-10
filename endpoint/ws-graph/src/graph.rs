use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};

use futures::{Stream, StreamExt as _};
use hyveos_sdk::{
    services::{debug::MeshTopologyEvent, NeighbourEvent},
    PeerId,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Debug, Default)]
struct Graph {
    nodes: HashMap<PeerId, HashSet<PeerId>>,
}

impl Graph {
    fn process_event(&mut self, MeshTopologyEvent { peer_id, event }: MeshTopologyEvent) {
        match event {
            NeighbourEvent::Init(neighbours) => self.process_init(peer_id, neighbours),
            NeighbourEvent::Lost(neighbour) => self.process_lost(peer_id, neighbour),
            NeighbourEvent::Discovered(neighbour) => self.process_discovered(peer_id, neighbour),
        }
    }

    fn process_init(&mut self, peer_id: PeerId, neighbours: Vec<PeerId>) {
        self.nodes.entry(peer_id).or_default().extend(neighbours);
    }

    fn process_discovered(&mut self, peer_id: PeerId, neighbour: PeerId) {
        self.nodes.entry(peer_id).or_default().insert(neighbour);
    }

    fn process_lost(&mut self, peer_id: PeerId, neighbour: PeerId) {
        self.nodes.entry(peer_id).or_default().remove(&neighbour);
        if self.nodes[&peer_id].is_empty() {
            self.nodes.remove(&peer_id);
        }
    }

    fn export(&self) -> ExportGraph {
        let nodes = self.nodes.keys().copied().map(ExportNode::from).collect();
        let links = self
            .nodes
            .iter()
            .flat_map(|(source, targets)| {
                targets
                    .iter()
                    .filter(|target| self.nodes.contains_key(target))
                    .map(move |target| ExportLink {
                        source: *source,
                        target: *target,
                    })
            })
            .collect();
        ExportGraph { nodes, links }
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
pub(crate) struct ExportNode {
    pub(crate) peer_id: PeerId,
}

impl From<PeerId> for ExportNode {
    fn from(peer_id: PeerId) -> Self {
        Self { peer_id }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExportLink {
    pub(crate) source: PeerId,
    pub(crate) target: PeerId,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExportGraph {
    pub(crate) nodes: HashSet<ExportNode>,
    pub(crate) links: Vec<ExportLink>,
}

pub(crate) type ParallelGraphReceiver = (ExportGraph, broadcast::Receiver<Arc<ExportGraph>>);

pub(crate) struct ParallelGraph<T> {
    graph: Graph,
    broadcast: broadcast::Sender<Arc<ExportGraph>>,
    subscription: mpsc::Receiver<oneshot::Sender<ParallelGraphReceiver>>,
    stream: T,
}

#[derive(Clone)]
pub(crate) struct ParallelGraphClient {
    sender: mpsc::Sender<oneshot::Sender<ParallelGraphReceiver>>,
}

impl ParallelGraphClient {
    pub(crate) async fn subscribe(&mut self) -> anyhow::Result<ParallelGraphReceiver> {
        let (sender, receiver) = oneshot::channel();
        self.sender.send(sender).await?;
        Ok(receiver.await?)
    }
}

pub(crate) enum Void {}

impl<T, E> ParallelGraph<T>
where
    T: Stream<Item = Result<MeshTopologyEvent, E>> + Unpin,
    E: Error + Send + Sync + 'static,
{
    pub(crate) fn new(stream: T) -> (Self, ParallelGraphClient) {
        let (sender, receiver) = mpsc::channel(1);
        let (broadcast, _) = broadcast::channel(1);
        let graph = Self {
            graph: Graph::default(),
            broadcast,
            subscription: receiver,
            stream,
        };
        let client = ParallelGraphClient { sender };
        (graph, client)
    }

    async fn process_event(&mut self, event: MeshTopologyEvent) {
        self.graph.process_event(event);
        let export_graph = self.graph.export();

        // We can ignore the error here,
        // because if it fails it means no one is listening
        // which is fine
        let _ = self.broadcast.send(Arc::new(export_graph));
    }

    fn handle_subscription(&self, sender: oneshot::Sender<ParallelGraphReceiver>) {
        let _ = sender.send((self.graph.export(), self.broadcast.subscribe()));
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Void> {
        loop {
            tokio::select! {
                event = self.stream.next() => {
                    let event = event.ok_or_else(|| anyhow::anyhow!("stream closed"))??;
                    self.process_event(event).await;
                }
                sender = self.subscription.recv() => {
                    let sender = sender.ok_or_else(|| anyhow::anyhow!("sender closed"))?;
                    self.handle_subscription(sender);
                }
            }
        }
    }
}
