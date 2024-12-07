use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    future::Future as _,
    io,
    pin::Pin,
    sync::{Arc, PoisonError, RwLock},
    task::{ready, Context, Poll},
};

use batman_neighbours_core::{BatmanNeighbour, BatmanNeighboursServerClient, Error as BatmanError};
use futures::{stream::StreamExt as _, Stream as _};
use hyveos_ifaddr::IfAddr;
use hyveos_ifwatcher::{IfEvent, IfWatcher};
use hyveos_macaddress::MacAddress;
use itertools::Itertools as _;
use libp2p::{
    core::{transport::PortUse, Endpoint},
    swarm::{
        dial_opts::DialOpts, dummy, ConnectionDenied, ConnectionId, FromSwarm, ListenAddresses,
        NetworkBehaviour, THandler, THandlerInEvent, THandlerOutEvent, ToSwarm,
    },
    Multiaddr, PeerId,
};
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

use self::{
    resolver::NeighbourResolver,
    store::{NeighbourStore, ReadOnlyNeighbourStore},
};
use crate::{behaviour::store::NeighbourStoreUpdate, Config, Error, ResolvedNeighbour};

mod resolver;
pub mod store;

const DISCOVERED_NEIGHBOUR_CHANNEL_BUFFER: usize = 1;
const RESOLVED_NEIGHBOUR_CHANNEL_BUFFER: usize = 1;

#[derive(Debug, Clone)]
pub enum Event {
    NeighbourUpdate(NeighbourStoreUpdate),
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

    #[cfg(target_os = "linux")]
    async fn run_task(
        batman_if_index: u32,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        mut listen_addresses_receiver: watch::Receiver<()>,
    ) -> Result<Multiaddr, Error> {
        tracing::info!(if_index=%batman_if_index, "Waiting for Batman interface address");
        loop {
            tracing::info!(
                if_index=%batman_if_index,
                "Listen addresses: {:?}",
                listen_addresses.read().unwrap_or_else(PoisonError::into_inner)
            );
            if let Some(multiaddr) = listen_addresses
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .find(|&multiaddr| {
                    if let Ok(addr) = IfAddr::try_from(multiaddr) {
                        addr.if_index == batman_if_index
                    } else {
                        false
                    }
                })
                .cloned()
            {
                return Ok(multiaddr);
            }

            listen_addresses_receiver
                .changed()
                .await
                .map_err(|_| Error::ChannelClosed("ListenAddressesReceiver".into()))?;
        }
    }

    #[cfg(not(target_os = "linux"))]
    async fn run_task(
        batman_if_index: u32,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        mut listen_addresses_receiver: watch::Receiver<()>,
    ) -> Result<Multiaddr, Error> {
        loop {}
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
    pending_swarm_events: VecDeque<ToSwarm<Event, THandlerInEvent<Behaviour>>>,
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
            tokio::sync::mpsc::channel(DISCOVERED_NEIGHBOUR_CHANNEL_BUFFER);
        let (resolved_neighbour_sender, resolved_neighbour_receiver) =
            tokio::sync::mpsc::channel(RESOLVED_NEIGHBOUR_CHANNEL_BUFFER);

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
            neighbour_store: Arc::default(),
            pending_swarm_events: VecDeque::new(),
        })
    }

    #[allow(clippy::too_many_lines)]
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
            for addr in std::mem::take(&mut self.pending_neighbour_resolvers) {
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
            tracing::trace!("Received discovered neighbours");
            match result {
                Ok(Ok(neighbours)) => {
                    let mut update = self
                        .neighbour_store
                        .write()
                        .unwrap_or_else(PoisonError::into_inner)
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
                        if let Some((_, sender, addr)) = self.neighbour_resolvers.get_mut(&if_index)
                        {
                            let addr = addr.clone();
                            if let Some(discovered_neighbours) =
                                match sender.try_send(discovered_neighbours) {
                                    Ok(()) => None,
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
                                }
                            {
                                // undiscover neighbours, so we can rediscover them later
                                for mac in discovered_neighbours {
                                    update.discovered.remove(&mac);
                                    self.neighbour_store
                                        .write()
                                        .unwrap_or_else(PoisonError::into_inner)
                                        .unresolved
                                        .remove(&mac);
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
            let mut neighbour_store = self
                .neighbour_store
                .write()
                .unwrap_or_else(PoisonError::into_inner);

            match res {
                Ok(neighbour) => {
                    tracing::info!(peer=%neighbour.peer_id , "Resolved neighbour: {}", neighbour.direct_addr);

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

        if let Some(event) = self.pending_swarm_events.pop_front() {
            return Poll::Ready(event);
        }

        if neighbour_update.has_changes() {
            for ResolvedNeighbour {
                peer_id,
                batman_addr,
                direct_addr,
                ..
            } in neighbour_update.resolved.values()
            {
                self.pending_swarm_events.push_back(ToSwarm::Dial {
                    opts: DialOpts::peer_id(peer_id)
                        .addresses(vec![direct_addr.clone(), batman_addr.clone()])
                        .build(),
                });
            }
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
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .find(|&multiaddr| {
                    if let Ok(listen_addr) = IfAddr::try_from(multiaddr) {
                        listen_addr.if_index == addr.if_index
                    } else {
                        false
                    }
                })
                .cloned()
            {
                match NeighbourResolver::new(
                    config,
                    self.local_peer_id,
                    self.batman_addr.clone(),
                    direct_address,
                    addr.clone(),
                    self.resolved_neighbour_sender.clone(),
                ) {
                    Ok((resolver, sender)) => {
                        let (_, sender, _) =
                            entry.insert((tokio::spawn(resolver), sender, addr.clone()));

                        let unresolved_neighbours = self
                            .neighbour_store
                            .read()
                            .unwrap_or_else(PoisonError::into_inner)
                            .unresolved
                            .values()
                            .filter(|n| n.if_index == addr.if_index)
                            .map(|n| n.mac)
                            .collect::<Vec<_>>();

                        if !unresolved_neighbours.is_empty()
                            && sender.try_send(unresolved_neighbours).is_err()
                        {
                            tracing::error!(
                                "Failed to send unresolved neighbours to NEWLY CREATED resolver"
                            );
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
    #[must_use]
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

    pub fn get_neighbour_store(&self) -> Option<ReadOnlyNeighbourStore> {
        if let BehaviourState::ResolvingNeighbours(behaviour) = &self.state {
            Some(behaviour.neighbour_store.clone().into())
        } else {
            None
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

    #[tracing::instrument(skip(self))]
    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        addresses: &[Multiaddr],
        effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        tracing::info!(peer=?maybe_peer, "Handling pending outbound connection called");
        let (Some(peer_id), Some(store)) = (maybe_peer, self.get_neighbour_store()) else {
            return Ok(Vec::new());
        };

        tracing::info!(peer=%peer_id, "Handling pending outbound connection");

        let store = store.read();

        let Some(neighbours) = store.resolved.get(&peer_id) else {
            return Ok(Vec::new());
        };

        tracing::info!(peer=%peer_id, "Resolved neighbours for outbound connection: {:?}", neighbours);

        Ok(neighbours
            .values()
            .map(|n| &n.direct_addr)
            .chain(neighbours.values().map(|n| &n.batman_addr).take(1))
            .filter(|addr| addresses.iter().all(|a| addr != &a))
            .cloned()
            .collect())
    }

    fn handle_established_outbound_connection(
        &mut self,
        _: ConnectionId,
        _: PeerId,
        _: &Multiaddr,
        _: Endpoint,
        _: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        Ok(dummy::ConnectionHandler)
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        self.listen_addresses
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .on_swarm_event(&event);

        if matches!(
            event,
            FromSwarm::NewListenAddr(_) | FromSwarm::ExpiredListenAddr(_)
        ) && self.listen_addresses_notifier.send(()).is_err()
        {
            tracing::error!("Failed to notify listen addresses changed");
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        _: THandlerOutEvent<Self>,
    ) {
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        loop {
            match self.state {
                BehaviourState::GettingBatmanAddr(ref mut behaviour) => match behaviour.poll(cx) {
                    Poll::Ready(Ok(batman_addr)) => {
                        tracing::info!("Got Batman address: {}", batman_addr);

                        let behaviour = match ResolvingNeighboursBehaviour::new(
                            &self.config,
                            self.local_peer_id,
                            batman_addr,
                            self.listen_addresses.clone(),
                            self.listen_addresses_notifier.subscribe(),
                        ) {
                            Ok(behaviour) => behaviour,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to create ResolvingNeighboursBehaviour: {}",
                                    e
                                );
                                *behaviour = GettingBatmanAddrBehaviour::new(
                                    &self.config,
                                    self.listen_addresses.clone(),
                                    self.listen_addresses_notifier.subscribe(),
                                );
                                continue;
                            }
                        };

                        self.state = BehaviourState::ResolvingNeighbours(behaviour);
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
