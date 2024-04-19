use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Display,
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard},
};

use libp2p::PeerId;
use macaddress::MacAddress;

use crate::{ResolvedNeighbour, UnresolvedNeighbour};

#[derive(Debug, Clone, Default)]
pub struct NeighbourStoreUpdate {
    pub discovered: HashMap<MacAddress, UnresolvedNeighbour>,
    pub resolved: HashMap<MacAddress, ResolvedNeighbour>,
    pub lost_unresolved: HashMap<MacAddress, UnresolvedNeighbour>,
    pub lost_resolved: HashMap<MacAddress, ResolvedNeighbour>,
    pub lost_peers: HashSet<PeerId>,
}

impl Display for NeighbourStoreUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for mac in self.discovered.keys() {
            writeln!(f, "+ {mac}")?;
        }

        for neighbour in self.resolved.values() {
            writeln!(f, "+ {} ({})", neighbour.direct_addr, neighbour.peer_id)?;
        }

        for mac in self.lost_unresolved.keys() {
            writeln!(f, "- {mac}")?;
        }

        for neighbour in self.lost_resolved.values() {
            writeln!(f, "- {} ({})", neighbour.direct_addr, neighbour.peer_id)?;
        }

        for peer in &self.lost_peers {
            writeln!(f, "- {peer}")?;
        }

        Ok(())
    }
}

impl NeighbourStoreUpdate {
    pub fn combine(&mut self, other: NeighbourStoreUpdate) {
        self.discovered.extend(other.discovered);
        self.resolved.extend(other.resolved);
        self.lost_unresolved.extend(other.lost_unresolved);
        self.lost_resolved.extend(other.lost_resolved);
        self.lost_peers.extend(other.lost_peers);
    }

    pub fn has_changes(&self) -> bool {
        !(self.discovered.is_empty()
            && self.resolved.is_empty()
            && self.lost_unresolved.is_empty()
            && self.lost_resolved.is_empty()
            && self.lost_peers.is_empty())
    }
}

#[derive(Debug, Default)]
pub struct NeighbourStore {
    pub unresolved: HashMap<MacAddress, UnresolvedNeighbour>,
    pub resolved: HashMap<PeerId, HashMap<MacAddress, ResolvedNeighbour>>,
}

impl NeighbourStore {
    pub(super) fn update_available_neighbours(
        &mut self,
        neighbours: impl IntoIterator<Item = UnresolvedNeighbour>,
    ) -> NeighbourStoreUpdate {
        let mut discovered = HashMap::new();

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
            discovered.insert(mac, neighbour);
        }

        let mut lost_unresolved = HashMap::new();

        for mac in lost_unresolved_macs {
            let Some(neighbour) = self.unresolved.remove(&mac) else {
                continue;
            };

            lost_unresolved.insert(mac, neighbour);
        }

        let mut lost_resolved = HashMap::new();
        let mut lost_peers = HashSet::new();

        for (mac, id) in lost_resolved_macs {
            let Entry::Occupied(mut entry) = self.resolved.entry(id) else {
                continue;
            };

            let neighbours = entry.get_mut();
            let Some(neighbour) = neighbours.remove(&mac) else {
                continue;
            };

            lost_resolved.insert(mac, neighbour);

            if neighbours.is_empty() {
                lost_peers.insert(id);
                entry.remove();
            }
        }

        NeighbourStoreUpdate {
            discovered,
            resolved: HashMap::new(),
            lost_unresolved,
            lost_resolved,
            lost_peers,
        }
    }

    pub(super) fn resolve_neighbour(
        &mut self,
        neighbour: ResolvedNeighbour,
    ) -> NeighbourStoreUpdate {
        let mac = neighbour.mac;

        let mut discovered = HashMap::new();

        if self.unresolved.remove(&mac).is_none() {
            discovered.insert(
                mac,
                UnresolvedNeighbour {
                    if_index: neighbour.if_index,
                    mac,
                },
            );
        }

        self.resolved
            .entry(neighbour.peer_id)
            .or_default()
            .insert(mac, neighbour.clone());

        let mut resolved = HashMap::new();
        resolved.insert(mac, neighbour);

        NeighbourStoreUpdate {
            discovered,
            resolved,
            ..Default::default()
        }
    }

    pub(super) fn remove_neighbour(&mut self, mac: MacAddress) -> NeighbourStoreUpdate {
        if let Some(neighbour) = self.unresolved.remove(&mac) {
            let mut lost_unresolved = HashMap::new();
            lost_unresolved.insert(mac, neighbour);

            NeighbourStoreUpdate {
                lost_unresolved,
                ..Default::default()
            }
        } else {
            let mut lost_neighbour = None;
            let mut lost_peer = None;
            for (id, neighbours) in &mut self.resolved {
                if let Some(neighbour) = neighbours.remove(&mac) {
                    if neighbours.is_empty() {
                        lost_peer = Some(*id);
                    }

                    lost_neighbour = Some((neighbour.mac, neighbour));
                    break;
                }
            }

            if let Some(id) = lost_peer {
                self.resolved.remove(&id);
            }

            NeighbourStoreUpdate {
                lost_resolved: HashMap::from_iter(lost_neighbour),
                lost_peers: HashSet::from_iter(lost_peer),
                ..Default::default()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadOnlyNeighbourStore(Arc<RwLock<NeighbourStore>>);

impl ReadOnlyNeighbourStore {
    pub fn read(&self) -> RwLockReadGuard<NeighbourStore> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }
}

impl From<Arc<RwLock<NeighbourStore>>> for ReadOnlyNeighbourStore {
    fn from(store: Arc<RwLock<NeighbourStore>>) -> Self {
        Self(store)
    }
}
