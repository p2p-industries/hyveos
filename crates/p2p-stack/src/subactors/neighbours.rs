use std::{collections::HashMap, sync::Arc};

use libp2p::PeerId;
use libp2p_batman_adv::{
    Event as BatmanEvent, ReadOnlyNeighbourStore, ResolvedNeighbour, UnresolvedNeighbour,
};
use tokio::sync::{broadcast, oneshot};

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    impl_from_special_command,
};

#[derive(Debug)]
pub struct Actor {
    sender: broadcast::Sender<Arc<Event>>,
    neighbour_store: Option<ReadOnlyNeighbourStore>,
}

impl Default for Actor {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(10);
        Self {
            sender,
            neighbour_store: None,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Subscribe(oneshot::Sender<broadcast::Receiver<Arc<Event>>>),
    GetResolved(oneshot::Sender<HashMap<PeerId, Vec<ResolvedNeighbour>>>),
    GetUnresolved(oneshot::Sender<Vec<UnresolvedNeighbour>>),
}

impl_from_special_command!(Neighbours);

#[derive(Debug)]
pub enum Event {
    ResolvedNeighbour(ResolvedNeighbour),
    LostNeighbour(ResolvedNeighbour),
}

impl SubActor for Actor {
    type SubCommand = Command;
    type CommandError = void::Void;
    type Event = BatmanEvent;
    type EventError = void::Void;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Subscribe(sender) => {
                let _ = sender.send(self.sender.subscribe());
            }
            Command::GetResolved(sender) => {
                let neighbours = self
                    .get_neighbour_store(behaviour)
                    .map(|store| {
                        store
                            .read()
                            .resolved
                            .iter()
                            .map(|(id, v)| (*id, v.values().cloned().collect()))
                            .collect()
                    })
                    .unwrap_or_default();

                let _ = sender.send(neighbours);
            }
            Command::GetUnresolved(sender) => {
                let neighbours = self
                    .get_neighbour_store(behaviour)
                    .map(|store| store.read().unresolved.values().cloned().collect())
                    .unwrap_or_default();

                let _ = sender.send(neighbours);
            }
        }
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            BatmanEvent::NeighbourUpdate(update) => {
                for (_, neighbour) in update.resolved {
                    println!("Resolved neighbour: {:?}", neighbour);
                    behaviour
                        .kad
                        .add_address(&neighbour.peer_id, neighbour.batman_addr.clone());
                    behaviour.gossipsub.add_explicit_peer(&neighbour.peer_id);

                    let event = Event::ResolvedNeighbour(neighbour);

                    let _ = self.sender.send(Arc::new(event));
                }

                for (_, neighbour) in update.lost_resolved {
                    let event = Event::LostNeighbour(neighbour);

                    let _ = self.sender.send(Arc::new(event));
                }

                for peer_id in update.lost_peers {
                    behaviour.gossipsub.remove_explicit_peer(&peer_id);
                }
            }
        }
        Ok(())
    }
}

impl Actor {
    fn get_neighbour_store(
        &mut self,
        behaviour: &mut MyBehaviour,
    ) -> Option<ReadOnlyNeighbourStore> {
        if let Some(store) = &self.neighbour_store {
            Some(store.clone())
        } else if let Some(store) = behaviour.batman_neighbours.get_neighbour_store() {
            self.neighbour_store = Some(store.clone());
            Some(store)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn subscribe(&self) -> Result<broadcast::Receiver<Arc<Event>>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn get_resolved(
        &self,
    ) -> Result<HashMap<PeerId, Vec<ResolvedNeighbour>>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::GetResolved(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }

    pub async fn get_unresolved(&self) -> Result<Vec<UnresolvedNeighbour>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::GetUnresolved(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }
}
