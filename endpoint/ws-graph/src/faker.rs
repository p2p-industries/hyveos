use std::{
    cmp::min,
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

use futures::{Stream, StreamExt};
use p2p_industries_sdk::{
    services::{
        debug::{
            MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType, RequestDebugEvent,
            ResponseDebugEvent,
        },
        gossipsub::Message as GossipsubMessage,
        req_resp::{Request, Response, ResponseError},
        NeighbourEvent,
    },
    PeerId,
};
use rand::{
    distributions::{Alphanumeric, WeightedIndex},
    prelude::Distribution,
    seq::IteratorRandom,
    thread_rng, Rng,
};
use ulid::Ulid;

#[derive(Debug, Default)]
pub struct FakerOracle {
    links: HashMap<PeerId, HashSet<PeerId>>,
    queue: VecDeque<MeshTopologyEvent>,
    unanswered_requests: Vec<(Ulid, PeerId)>,
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

    fn message(&mut self, sender: PeerId) -> MessageDebugEvent {
        let message: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(thread_rng().gen_range(1..=100))
            .map(char::from)
            .collect();

        let neighbours = self.links.get(&sender).unwrap();
        if !neighbours.is_empty() && thread_rng().gen() {
            let receiver = neighbours
                .iter()
                .choose(&mut thread_rng())
                .copied()
                .unwrap();

            let req_id = Ulid::new();

            self.unanswered_requests.push((req_id, receiver));

            let topic = if thread_rng().gen_bool(0.9) {
                Some(
                    thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(thread_rng().gen_range(4..=10))
                        .map(char::from)
                        .collect(),
                )
            } else {
                None
            };

            MessageDebugEvent {
                sender,
                event: MessageDebugEventType::Request(RequestDebugEvent {
                    id: req_id,
                    receiver,
                    msg: Request {
                        data: message.into_bytes(),
                        topic,
                    },
                }),
            }
        } else {
            let topic = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(thread_rng().gen_range(4..=10))
                .map(char::from)
                .collect();

            MessageDebugEvent {
                sender,
                event: MessageDebugEventType::GossipSub(GossipsubMessage {
                    data: message.into_bytes(),
                    topic,
                }),
            }
        }
    }
}

impl Iterator for FakerOracle {
    type Item = Result<MeshTopologyEvent, MessageDebugEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.queue.pop_front() {
            return Some(Ok(event));
        }

        if thread_rng().gen() {
            if !self.unanswered_requests.is_empty() && thread_rng().gen() {
                let i = thread_rng().gen_range(0..self.unanswered_requests.len());
                let (req_id, sender) = self.unanswered_requests.remove(i);

                let response = if thread_rng().gen() {
                    let message: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(thread_rng().gen_range(1..=100))
                        .map(char::from)
                        .collect();
                    Response::Data(message.into_bytes())
                } else {
                    Response::Error(ResponseError::Script("Some error message".to_string()))
                };

                return Some(Err(MessageDebugEvent {
                    sender,
                    event: MessageDebugEventType::Response(ResponseDebugEvent { req_id, response }),
                }));
            } else if let Some(sender) = self.links.keys().choose(&mut thread_rng()).copied() {
                return Some(Err(self.message(sender)));
            }
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

pub(crate) fn faker_oracle(
) -> impl Stream<Item = Result<MeshTopologyEvent, MessageDebugEvent>> + Unpin + Send {
    let oracle = FakerOracle::default();
    Box::pin(futures::stream::iter(oracle).then(|event| async move {
        let duration = {
            let mut rng = thread_rng();
            Duration::from_millis(rng.gen_range(100..=5000))
        };
        tokio::time::sleep(duration).await;
        event
    }))
}
