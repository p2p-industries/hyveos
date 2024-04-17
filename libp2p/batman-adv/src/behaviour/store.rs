use std::collections::{hash_map::Entry, HashMap, HashSet};

use batman_neighbours_core::BatmanNeighbour;
use libp2p::PeerId;
use macaddress::MacAddress;

use crate::ResolvedNeighbour;

#[derive(Debug, Default)]
pub struct NeighbourStoreUpdate {
    pub discovered: Vec<BatmanNeighbour>,
    pub lost_unresolved: Vec<BatmanNeighbour>,
    pub lost_resolved: Vec<ResolvedNeighbour>,
    pub lost_peers: Vec<PeerId>,
}

impl NeighbourStoreUpdate {
    pub fn has_changes(&self) -> bool {
        !(self.discovered.is_empty()
            && self.lost_unresolved.is_empty()
            && self.lost_resolved.is_empty()
            && self.lost_peers.is_empty())
    }
}

#[derive(Default)]
pub struct NeighbourStore {
    pub unresolved: HashMap<MacAddress, BatmanNeighbour>,
    pub resolved: HashMap<PeerId, HashMap<MacAddress, ResolvedNeighbour>>,
}

impl NeighbourStore {
    pub fn update_available_neighbours(
        &mut self,
        neighbours: impl IntoIterator<Item = BatmanNeighbour>,
    ) -> NeighbourStoreUpdate {
        let mut discovered = Vec::new();

        let mut lost_unresolved_macs = self.unresolved.keys().copied().collect::<HashSet<_>>();
        let mut lost_resolved_macs = self
            .resolved
            .iter()
            .flat_map(|(id, n)| n.keys().map(|mac| (*mac, *id)))
            .collect::<HashMap<_, _>>();

        for neighbour in neighbours {
            let mac = neighbour.mac;

            if lost_unresolved_macs.remove(&mac) {
                continue;
            }

            if lost_resolved_macs.remove(&mac).is_some() {
                continue;
            }

            self.unresolved.insert(mac, neighbour.clone());
            discovered.push(neighbour);
        }

        let mut lost_unresolved = Vec::new();

        for mac in lost_unresolved_macs {
            let Some(neighbour) = self.unresolved.remove(&mac) else {
                continue;
            };

            lost_unresolved.push(neighbour);
        }

        let mut lost_resolved = Vec::new();
        let mut lost_peers = Vec::new();

        for (mac, id) in lost_resolved_macs {
            let Entry::Occupied(mut entry) = self.resolved.entry(id) else {
                continue;
            };

            let neighbours = entry.get_mut();
            let Some(neighbour) = neighbours.remove(&mac) else {
                continue;
            };

            lost_resolved.push(neighbour);

            if neighbours.is_empty() {
                lost_peers.push(id);
                entry.remove();
            }
        }

        NeighbourStoreUpdate {
            discovered,
            lost_unresolved,
            lost_resolved,
            lost_peers,
        }
    }

    pub fn resolve_neighbour(&mut self, neighbour: ResolvedNeighbour) {
        let mac = neighbour.mac;
        self.unresolved.remove(&mac);
        self.resolved
            .entry(neighbour.peer_id)
            .or_default()
            .insert(mac, neighbour);
    }

    pub fn remove_neighbour(&mut self, mac: MacAddress) -> NeighbourStoreUpdate {
        if let Some(neighbour) = self.unresolved.remove(&mac) {
            NeighbourStoreUpdate {
                lost_unresolved: vec![neighbour],
                ..Default::default()
            }
        } else {
            let mut lost_neighbour = None;
            let mut lost_peer = None;
            for (id, neighbours) in self.resolved.iter_mut() {
                if let Some(neighbour) = neighbours.remove(&mac) {
                    if neighbours.is_empty() {
                        lost_peer = Some(*id);
                    }

                    lost_neighbour = Some(neighbour);
                    break;
                }
            }

            if let Some(id) = lost_peer {
                self.resolved.remove(&id);
            }

            NeighbourStoreUpdate {
                lost_resolved: Vec::from_iter(lost_neighbour),
                lost_peers: Vec::from_iter(lost_peer),
                ..Default::default()
            }
        }
    }
}
