use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use batman_neighbours_core::{BatmanNeighbour, BatmanNeighboursServerClient};
use libp2p::{
    core::Endpoint,
    swarm::{
        dummy, ConnectionDenied, ConnectionId, FromSwarm, NetworkBehaviour, THandler,
        THandlerInEvent, THandlerOutEvent, ToSwarm,
    },
    Multiaddr, PeerId,
};
use macaddress::MacAddress;
use tarpc::{
    client::{self, RpcError},
    context,
    tokio_serde::formats::Bincode,
};
use tokio::sync::mpsc::{error::TrySendError, Receiver};

use crate::Config;

#[derive(Debug, Clone)]
pub enum Event {
    NeighboursChanged {
        discovered: Vec<BatmanNeighbour>,
        lost: Vec<MacAddress>,
    },
}

pub struct Behaviour {
    config: Config,
    neighbour_receiver: Receiver<Result<Result<Vec<BatmanNeighbour>, String>, RpcError>>,
    unresolved_neighbours: Arc<RwLock<HashMap<MacAddress, BatmanNeighbour>>>,
}

impl Behaviour {
    pub fn new(config: Config) -> Self {
        let (neighbour_sender, neighbour_receiver) = tokio::sync::mpsc::channel(1);

        let if_name = config.interface.clone();
        let socket_path = config.socket_path.clone();

        tokio::spawn(async move {
            let transport =
                tarpc::serde_transport::unix::connect(socket_path.as_ref(), Bincode::default)
                    .await
                    .unwrap();

            let client =
                BatmanNeighboursServerClient::new(client::Config::default(), transport).spawn();

            loop {
                let neighbours = client
                    .get_neighbours(context::current(), if_name.clone())
                    .await;

                if matches!(
                    neighbour_sender.try_send(neighbours),
                    Err(TrySendError::Closed(_))
                ) {
                    break;
                }

                tokio::time::sleep(config.refresh_interval).await;
            }
        });

        Self {
            config,
            neighbour_receiver,
            unresolved_neighbours: Default::default(),
        }
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = dummy::ConnectionHandler;
    type ToSwarm = Event;

    fn handle_established_inbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        println!("on_swarm_event: {:?}", event);
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        ev: THandlerOutEvent<Self>,
    ) {
        void::unreachable(ev)
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        while let Poll::Ready(Some(result)) = self.neighbour_receiver.poll_recv(cx) {
            match result {
                Ok(Ok(neighbours)) => {
                    let mut discovered_neighbours = Vec::new();

                    let mut unresolved_neighbours = self.unresolved_neighbours.write().unwrap();
                    let mut lost_neighbours =
                        HashSet::<MacAddress>::from_iter(unresolved_neighbours.keys().cloned());

                    for neighbour in neighbours
                        .into_iter()
                        .filter(|n| n.last_seen < self.config.neighbour_timeout)
                    {
                        let mac = neighbour.mac;
                        lost_neighbours.remove(&mac);

                        match unresolved_neighbours.entry(mac) {
                            Entry::Occupied(mut entry) => {
                                entry.insert(neighbour.clone());
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(neighbour.clone());
                                discovered_neighbours.push(neighbour);
                            }
                        }
                    }

                    for mac in lost_neighbours.iter() {
                        unresolved_neighbours.remove(mac);
                    }

                    if !discovered_neighbours.is_empty() || !lost_neighbours.is_empty() {
                        return Poll::Ready(ToSwarm::GenerateEvent(Event::NeighboursChanged {
                            discovered: discovered_neighbours,
                            lost: lost_neighbours.into_iter().collect(),
                        }));
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to get neighbours: {}", e);
                }
                Err(e) => {
                    eprintln!("Failed to receive neighbours: {}", e);
                }
            }
        }

        Poll::Pending
    }
}
