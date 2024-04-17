mod if_watcher;
mod resolver;
mod store;

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    future::Future as _,
    io,
    net::{IpAddr, Ipv6Addr},
    pin::Pin,
    sync::{Arc, RwLock},
    task::{ready, Context, Poll},
};

use batman_neighbours_core::{BatmanNeighbour, BatmanNeighboursServerClient, Error as BatmanError};
use futures::{
    stream::{self, StreamExt as _, TryStreamExt as _},
    Stream as _,
};
use itertools::Itertools as _;
use libp2p::{
    core::Endpoint,
    multiaddr::Protocol,
    swarm::{
        dummy, ConnectionDenied, ConnectionId, FromSwarm, ListenAddresses, NetworkBehaviour,
        THandler, THandlerInEvent, THandlerOutEvent, ToSwarm,
    },
    Multiaddr, PeerId,
};
use macaddress::MacAddress;
use netlink_packet_route::address::AddressAttribute;
use tarpc::{
    client::{self, RpcError},
    context,
    tokio_serde::formats::Bincode,
};
use tokio::{
    sync::{
        mpsc::{error::TrySendError, Receiver, Sender},
        oneshot, watch,
    },
    task::JoinHandle,
};
use tokio_stream::wrappers::WatchStream;

use crate::{behaviour::store::NeighbourStoreUpdate, Config, Error, ResolvedNeighbour};

use self::{
    if_watcher::{IfAddr, IfEvent, IfWatcher},
    resolver::NeighbourResolver,
    store::NeighbourStore,
};

#[derive(Debug, Clone)]
pub enum Event {
    NeighbourUpdate(NeighbourStoreUpdate),
}

trait ListenAddressesExt {
    fn find_address(&self, p: impl FnMut(&Ipv6Addr) -> bool) -> Option<Multiaddr>;
}

impl ListenAddressesExt for ListenAddresses {
    fn find_address(&self, mut p: impl FnMut(&Ipv6Addr) -> bool) -> Option<Multiaddr> {
        self.iter().find_map(|multiaddr| {
            multiaddr.iter().find_map(|segment| {
                if let Protocol::Ip6(addr) = segment {
                    if p(&addr) {
                        Some(multiaddr.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }
}

struct GettingBatmanAddrBehaviour {
    receiver: oneshot::Receiver<Result<Multiaddr, Error>>,
}

impl GettingBatmanAddrBehaviour {
    fn new(
        config: &Config,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        listen_addresses_receiver: watch::Receiver<()>,
    ) -> Self {
        let (sender, receiver) = oneshot::channel();

        let batman_if_index = config.batman_if_index;

        tokio::spawn(async move {
            let res =
                Self::run_task(batman_if_index, listen_addresses, listen_addresses_receiver).await;

            if sender.send(res).is_err() {
                tracing::error!("Failed to send Batman address");
            }
        });

        Self { receiver }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<Multiaddr, Error>> {
        match ready!(Pin::new(&mut self.receiver).poll(cx)) {
            Ok(res) => Poll::Ready(res),
            Err(_) => Poll::Ready(Err(Error::ChannelClosed(
                "GettingBatmanAddrBehaviour".into(),
            ))),
        }
    }

    async fn run_task(
        batman_if_index: u32,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        mut listen_addresses_receiver: watch::Receiver<()>,
    ) -> Result<Multiaddr, Error> {
        let (conn, handle, _) = rtnetlink::new_connection()
            .map_err(|e| Error::CreateNetlinkConnection(e.to_string()))?;
        tokio::spawn(conn);

        let addresses = handle
            .address()
            .get()
            .set_link_index_filter(batman_if_index)
            .execute()
            .map_err(|e| Error::GetAddresses(e.to_string()))
            .map_ok(|msg| stream::iter(msg.attributes).map(Ok::<_, Error>))
            .try_flatten()
            .try_filter_map(|attr| async move {
                Ok(if let AddressAttribute::Address(IpAddr::V6(local)) = attr {
                    Some(local)
                } else {
                    None
                })
            })
            .try_collect::<HashSet<_>>()
            .await?;

        loop {
            if let Some(multiaddr) = listen_addresses
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .find_address(|addr| addresses.contains(addr))
            {
                return Ok(multiaddr);
            }

            listen_addresses_receiver
                .changed()
                .await
                .map_err(|_| Error::ChannelClosed("ListenAddressesReceiver".into()))?;
        }
    }
}

struct ResolvingNeighboursBehaviour {
    local_peer_id: PeerId,
    batman_addr: Multiaddr,
    if_watch: IfWatcher,
    listen_addresses: Arc<RwLock<ListenAddresses>>,
    listen_addresses_notifier: WatchStream<()>,
    pending_neighbour_resolvers: HashSet<IfAddr>,
    #[allow(clippy::type_complexity)]
    neighbour_resolvers: HashMap<u32, (JoinHandle<()>, Sender<Vec<MacAddress>>, IfAddr)>,
    discovered_neighbour_receiver:
        Receiver<Result<Result<Vec<BatmanNeighbour>, BatmanError>, RpcError>>,
    resolved_neighbour_sender: Sender<Result<ResolvedNeighbour, MacAddress>>,
    resolved_neighbour_receiver: Receiver<Result<ResolvedNeighbour, MacAddress>>,
    neighbour_store: Arc<RwLock<NeighbourStore>>,
}

impl ResolvingNeighboursBehaviour {
    fn new(
        config: &Config,
        local_peer_id: PeerId,
        batman_addr: Multiaddr,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        listen_addresses_notifier: watch::Receiver<()>,
    ) -> io::Result<Self> {
        let (discovered_neighbour_sender, discovered_neighbour_receiver) =
            tokio::sync::mpsc::channel(1);
        let (resolved_neighbour_sender, resolved_neighbour_receiver) =
            tokio::sync::mpsc::channel(1);

        let batman_if_index = config.batman_if_index;
        let socket_path = config.socket_path.clone();
        let refresh_interval = config.refresh_interval;

        tokio::spawn(async move {
            let Ok(transport) =
                tarpc::serde_transport::unix::connect(socket_path.as_ref(), Bincode::default).await
            else {
                return;
            };

            let client =
                BatmanNeighboursServerClient::new(client::Config::default(), transport).spawn();

            loop {
                let neighbours = client
                    .get_neighbours(context::current(), batman_if_index)
                    .await;

                if matches!(
                    discovered_neighbour_sender.try_send(neighbours),
                    Err(TrySendError::Closed(_))
                ) {
                    break;
                }

                tokio::time::sleep(refresh_interval).await;
            }
        });

        Ok(Self {
            local_peer_id,
            batman_addr,
            if_watch: IfWatcher::new()?,
            listen_addresses,
            listen_addresses_notifier: WatchStream::new(listen_addresses_notifier),
            pending_neighbour_resolvers: HashSet::new(),
            neighbour_resolvers: HashMap::new(),
            discovered_neighbour_receiver,
            resolved_neighbour_sender,
            resolved_neighbour_receiver,
            neighbour_store: Default::default(),
        })
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        config: &Config,
    ) -> Poll<ToSwarm<Event, THandlerInEvent<Behaviour>>> {
        if self
            .listen_addresses_notifier
            .poll_next_unpin(cx)
            .is_ready()
        {
            tracing::info!("Listen addresses changed");
            let mut pending_resolvers = HashSet::new();
            std::mem::swap(
                &mut self.pending_neighbour_resolvers,
                &mut pending_resolvers,
            );
            for addr in pending_resolvers {
                self.add_resolver(config.clone(), addr);
            }
        }

        while let Poll::Ready(Some(event)) = Pin::new(&mut self.if_watch).poll_next(cx) {
            tracing::info!("Got if event: {:?}", event);
            match event {
                Ok(IfEvent::Up(addr)) => {
                    if addr.if_index == config.batman_if_index || addr.addr.is_loopback() {
                        continue;
                    }

                    self.add_resolver(config.clone(), addr);
                }
                Ok(IfEvent::Down(addr)) => {
                    if let Some((handle, _, _)) = self.neighbour_resolvers.remove(&addr.if_index) {
                        handle.abort();
                    } else {
                        self.pending_neighbour_resolvers.remove(&addr);
                    }
                }
                Err(err) => tracing::error!("if watch returned an error: {}", err),
            }
        }

        let mut neighbour_update = NeighbourStoreUpdate::default();

        if let Poll::Ready(Some(result)) = self.discovered_neighbour_receiver.poll_recv(cx) {
            tracing::info!("Received discovered neighbours");
            match result {
                Ok(Ok(neighbours)) => {
                    let mut update = self.neighbour_store.write().unwrap_or_else(|e| e.into_inner())
                        .update_available_neighbours(
                            neighbours
                                .into_iter()
                                .filter(|n| n.last_seen < config.neighbour_timeout)
                                .map(Into::into),
                        );

                    for (if_index, discovered_neighbours) in update
                        .discovered
                        .iter()
                        .map(|(mac, n)| (n.if_index, *mac))
                        .into_group_map()
                    {
                        if let Some((_, sender, addr)) = self.neighbour_resolvers.get_mut(&if_index) {
                            let addr = *addr;
                            if let Some(discovered_neighbours) = match sender.try_send(discovered_neighbours) {
                                Ok(_) => None,
                                Err(TrySendError::Closed(discovered_neighbours)) => {
                                    tracing::error!("Neighbour resolver dropped");
                                    self.neighbour_resolvers.remove(&if_index);
                                    self.add_resolver(config.clone(), addr);
                                    Some(discovered_neighbours)
                                }
                                Err(TrySendError::Full(discovered_neighbours)) => {
                                    tracing::error!("Neighbour resolver buffer full");
                                    Some(discovered_neighbours)
                                }
                            } {
                                // undiscover neighbours, so we can rediscover them later
                                for mac in discovered_neighbours {
                                    update.discovered.remove(&mac);
                                    self.neighbour_store.write().unwrap_or_else(|e| e.into_inner()).unresolved.remove(&mac);
                                }
                            }
                        }
                    }

                    neighbour_update = update;
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to get neighbours: {}", e);
                }
                Err(e) => {
                    tracing::error!("Failed to receive neighbours: {}", e);
                }
            }
        }

        while let Poll::Ready(Some(res)) = self.resolved_neighbour_receiver.poll_recv(cx) {
            let mut neighbour_store = self.neighbour_store.write().unwrap_or_else(|e| e.into_inner());

            match res {
                Ok(neighbour) => {
                    tracing::info!(peer=%neighbour.peer_id , "\nResolved neighbour: {}", neighbour.direct_addr);

                    let update = neighbour_store.resolve_neighbour(neighbour);

                    neighbour_update.combine(update);
                }
                Err(mac) => {
                    tracing::info!("Failed to resolve neighbour: {}", mac);

                    let update = neighbour_store.remove_neighbour(mac);

                    neighbour_update.combine(update);
                }
            }
        }

        if neighbour_update.has_changes() {
            return Poll::Ready(ToSwarm::GenerateEvent(Event::NeighbourUpdate(
                neighbour_update,
            )));
        }

        Poll::Pending
    }

    fn add_resolver(&mut self, config: Config, addr: IfAddr) {
        if let Entry::Vacant(entry) = self.neighbour_resolvers.entry(addr.if_index) {
            if let Some(direct_address) = self
                .listen_addresses
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .find_address(|a| a == &addr.addr)
            {
                match NeighbourResolver::new(
                    config,
                    self.local_peer_id,
                    self.batman_addr.clone(),
                    direct_address,
                    addr,
                    self.resolved_neighbour_sender.clone(),
                ) {
                    Ok((resolver, sender)) => {
                        let (_, sender, _) = entry.insert((tokio::spawn(resolver), sender, addr));

                        let unresolved_neighbours = self
                            .neighbour_store
                            .read()
                            .unwrap_or_else(|e| e.into_inner())
                            .unresolved
                            .values()
                            .filter(|n| n.if_index == addr.if_index)
                            .map(|n| n.mac)
                            .collect::<Vec<_>>();

                        if !unresolved_neighbours.is_empty() && sender.try_send(unresolved_neighbours).is_err() {
                            tracing::error!("Failed to send unresolved neighbours to NEWLY CREATED resolver");
                        }
                    }
                    Err(e) => tracing::error!("Failed to create neighbour resolver: {}", e),
                }
            } else {
                self.pending_neighbour_resolvers.insert(addr);
            }
        }
    }
}

enum BehaviourState {
    GettingBatmanAddr(GettingBatmanAddrBehaviour),
    ResolvingNeighbours(ResolvingNeighboursBehaviour),
}

pub struct Behaviour {
    config: Config,
    local_peer_id: PeerId,
    listen_addresses: Arc<RwLock<ListenAddresses>>,
    listen_addresses_notifier: watch::Sender<()>,
    state: BehaviourState,
}

impl Behaviour {
    pub fn new(config: Config, local_peer_id: PeerId) -> Self {
        let listen_addresses = Arc::new(RwLock::new(ListenAddresses::default()));

        let (listen_addresses_notifier, listen_addresses_receiver) = watch::channel(());

        let state = BehaviourState::GettingBatmanAddr(GettingBatmanAddrBehaviour::new(
            &config,
            listen_addresses.clone(),
            listen_addresses_receiver,
        ));

        Self {
            config,
            local_peer_id,
            listen_addresses,
            listen_addresses_notifier,
            state,
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
        self.listen_addresses
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .on_swarm_event(&event);

        if matches!(
            event,
            FromSwarm::NewListenAddr(_) | FromSwarm::ExpiredListenAddr(_)
        ) && self.listen_addresses_notifier.send(()).is_err() {
            tracing::error!("Failed to notify listen addresses changed");
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        ev: THandlerOutEvent<Self>,
    ) {
        void::unreachable(ev)
    }

    #[allow(clippy::expect_used)]
    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        loop {
            match self.state {
                BehaviourState::GettingBatmanAddr(ref mut behaviour) => match behaviour.poll(cx) {
                    Poll::Ready(Ok(batman_addr)) => {
                        tracing::info!("Got Batman address: {}", batman_addr);

                        self.state = BehaviourState::ResolvingNeighbours(
                            ResolvingNeighboursBehaviour::new(
                                &self.config,
                                self.local_peer_id,
                                batman_addr,
                                self.listen_addresses.clone(),
                                self.listen_addresses_notifier.subscribe(),
                            )
                            .expect("Failed to create ResolvingNeighboursBehaviour"),
                        );
                    }
                    Poll::Ready(Err(e)) => {
                        tracing::error!("Failed to get Batman address: {}", e);
                        *behaviour = GettingBatmanAddrBehaviour::new(
                            &self.config,
                            self.listen_addresses.clone(),
                            self.listen_addresses_notifier.subscribe(),
                        );
                        continue;
                    }
                    Poll::Pending => return Poll::Pending,
                },
                BehaviourState::ResolvingNeighbours(ref mut behaviour) => {
                    return behaviour.poll(cx, &self.config);
                }
            }
        }
    }
}
