use std::{
    cmp::min,
    collections::{HashMap, HashSet, VecDeque},
};

use futures::{Stream, StreamExt};
use p2p_industries_sdk::{
    services::{debug::MeshTopologyEvent, NeighbourEvent},
    PeerId,
};
use rand::{distributions::WeightedIndex, prelude::Distribution, seq::IteratorRandom, thread_rng};

#[derive(Debug, Default)]
pub struct FakerOracle {
    links: HashMap<PeerId, HashSet<PeerId>>,
    queue: VecDeque<MeshTopologyEvent>,
}

impl FakerOracle {
    fn connnect(&mut self, a: PeerId, b: PeerId) {
        if self
            .links
            .get_mut(&a)
            .expect("There should be a HashSet for that node, this is a consistency error")
            .insert(b)
        {
            self.queue.push_back(MeshTopologyEvent {
                peer_id: a,
                event: NeighbourEvent::Discovered(b),
            });
        }
    }

    fn connect_bidirectional(&mut self, a: PeerId, b: PeerId) {
        self.connnect(a, b);
        self.connnect(b, a);
    }

    fn disconnect(&mut self, a: PeerId, b: PeerId) {
        if self
            .links
            .get_mut(&a)
            .expect("There should be a HashSet for that node, this is a consistency error")
            .remove(&b)
        {
            self.queue.push_back(MeshTopologyEvent {
                peer_id: a,
                event: NeighbourEvent::Lost(b),
            });
        } else {
            panic!("There should be a link between a and b, this is a consistency error");
        }
        if self.links.get(&a).unwrap().is_empty()
            && !self
                .links
                .values()
                .any(|neighbours| neighbours.contains(&a))
        {
            self.links.remove(&a);
        }
    }

    fn add_node(&mut self) {
        let peer_id = PeerId::random();
        let neighbours = self.links.keys().choose(&mut thread_rng()).copied();
        self.links
            .insert(peer_id, neighbours.iter().copied().collect());
        self.queue.push_back(MeshTopologyEvent {
            peer_id,
            event: NeighbourEvent::Init(neighbours.iter().copied().collect()),
        });
        for neighbour in neighbours.iter() {
            self.connnect(*neighbour, peer_id);
        }
    }

    fn add_link(&mut self) {
        let a = self.links.keys().choose(&mut thread_rng()).copied();
        let b = self.links.keys().choose(&mut thread_rng()).copied();
        if let (Some(a), Some(b)) = (a, b) {
            self.connect_bidirectional(a, b);
        }
    }

    fn remove_node(&mut self) {
        if let Some(peer_id) = self.links.keys().choose(&mut thread_rng()).copied() {
            let neighbours = self.links.remove(&peer_id).unwrap();
            for neighbour in neighbours {
                if !self.links.contains_key(&neighbour) {
                    continue;
                }
                self.disconnect(neighbour, peer_id);
            }
        }
    }

    fn remove_link(&mut self) {
        if let Some((peer_id, neighbours)) = self.links.iter().choose(&mut thread_rng()) {
            let peer_id = *peer_id;
            if let Some(neighbour) = neighbours.iter().choose(&mut thread_rng()).copied() {
                self.disconnect(peer_id, neighbour);
                if self
                    .links
                    .get(&neighbour)
                    .map_or(false, |e| e.contains(&peer_id))
                {
                    self.disconnect(neighbour, peer_id);
                }
            }
        }
    }
}

impl Iterator for FakerOracle {
    type Item = MeshTopologyEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.queue.pop_front() {
            return Some(event);
        }

        const TARGET_SIZE: usize = 100;
        let len = self.links.len();
        let add_weight = TARGET_SIZE.saturating_sub(len);
        let rm_weight = min(len, TARGET_SIZE);
        let weights: [usize; 4] = [add_weight, add_weight, rm_weight, rm_weight];
        let dist = WeightedIndex::new(weights).unwrap();

        match dist.sample(&mut thread_rng()) % 4 {
            0 => self.add_node(),
            1 => self.add_link(),
            2 => self.remove_node(),
            3 => self.remove_link(),
            _ => unreachable!(),
        }
        self.next()
    }
}

pub(crate) fn faker_oracle() -> impl Stream<Item = MeshTopologyEvent> + Unpin {
    let oracle = FakerOracle::default();
    Box::pin(futures::stream::iter(oracle).then(|event| async move {
        tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        event
    }))
}
