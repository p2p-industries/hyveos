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

use batman_neighbours_core::{BatmanNeighbour, BatmanNeighboursServerClient};
use futures::{
    stream::{self, StreamExt as _, TryStreamExt as _},
    Stream as _,
};
use itertools::Itertools;
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
use rtnetlink::Error as RtnetlinkError;
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

use crate::{Config, ResolvedNeighbour};

use self::{
    if_watcher::{IfAddr, IfEvent, IfWatcher},
    resolver::NeighbourResolver,
    store::NeighbourStore,
};

#[derive(Debug, Clone)]
pub enum Event {}

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
    receiver: oneshot::Receiver<anyhow::Result<Multiaddr>>,
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

            sender.send(res).unwrap();
        });

        Self { receiver }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<Multiaddr, anyhow::Error>> {
        match ready!(Pin::new(&mut self.receiver).poll(cx)) {
            Ok(res) => Poll::Ready(res),
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }

    async fn run_task(
        batman_if_index: u32,
        listen_addresses: Arc<RwLock<ListenAddresses>>,
        mut listen_addresses_receiver: watch::Receiver<()>,
    ) -> anyhow::Result<Multiaddr> {
        let (conn, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(conn);

        let addresses = handle
            .address()
            .get()
            .set_link_index_filter(batman_if_index)
            .execute()
            .map_ok(|msg| stream::iter(msg.attributes).map(Ok::<_, RtnetlinkError>))
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
                .unwrap()
                .find_address(|addr| addresses.contains(addr))
            {
                return Ok(multiaddr);
            }

            listen_addresses_receiver.changed().await?;
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
    neighbour_resolvers: HashMap<u32, (JoinHandle<()>, Sender<Vec<MacAddress>>)>,
    discovered_neighbour_receiver: Receiver<Result<Result<Vec<BatmanNeighbour>, String>, RpcError>>,
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
            let transport =
                tarpc::serde_transport::unix::connect(socket_path.as_ref(), Bincode::default)
                    .await
                    .unwrap();

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
        println!("Polling ResolvingNeighboursBehaviour");

        if self
            .listen_addresses_notifier
            .poll_next_unpin(cx)
            .is_ready()
        {
            println!("Listen addresses changed");
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
            println!("Got if event: {:?}", event);
            match event {
                Ok(IfEvent::Up(addr)) => {
                    if addr.if_index == config.batman_if_index || addr.addr.is_loopback() {
                        continue;
                    }

                    self.add_resolver(config.clone(), addr);
                }
                Ok(IfEvent::Down(addr)) => {
                    if let Some((handle, _)) = self.neighbour_resolvers.remove(&addr.if_index) {
                        handle.abort();
                    } else {
                        self.pending_neighbour_resolvers.remove(&addr);
                    }
                }
                Err(err) => eprintln!("if watch returned an error: {}", err),
            }
        }

        while let Poll::Ready(Some(result)) = self.discovered_neighbour_receiver.poll_recv(cx) {
            println!("Received discovered neighbours");
            match result {
                Ok(Ok(neighbours)) => {
                    let update = self
                        .neighbour_store
                        .write()
                        .unwrap()
                        .update_available_neighbours(
                            neighbours
                                .into_iter()
                                .filter(|n| n.last_seen < config.neighbour_timeout),
                        );

                    for (if_index, discovered_neighbours) in update
                        .discovered
                        .iter()
                        .map(|n| (n.if_index, n.mac))
                        .into_group_map()
                    {
                        if let Some((_, sender)) = self.neighbour_resolvers.get_mut(&if_index) {
                            sender.try_send(discovered_neighbours).unwrap();
                        }
                    }

                    // TODO: do something with the update
                    if update.has_changes() {
                        println!("\nUpdate: {:?}", update);
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

        while let Poll::Ready(Some(res)) = self.resolved_neighbour_receiver.poll_recv(cx) {
            println!("locking neighbour store");
            let mut neighbour_store = self.neighbour_store.write().unwrap();

            println!("locked neighbour store");

            match res {
                Ok(neighbour) => {
                    println!("\nResolved neighbour: {:#?}", neighbour);

                    neighbour_store.resolve_neighbour(neighbour);

                    // TODO: do something with the resolved neighbour
                }
                Err(mac) => {
                    eprintln!("Failed to resolve neighbour: {}", mac);

                    let update = neighbour_store.remove_neighbour(mac);

                    // TODO: do something with the update
                    println!("\nUpdate: {:#?}", update);
                }
            }
        }

        Poll::Pending
    }

    fn add_resolver(&mut self, config: Config, addr: IfAddr) {
        if let Entry::Vacant(entry) = self.neighbour_resolvers.entry(addr.if_index) {
            if let Some(direct_address) = self
                .listen_addresses
                .read()
                .unwrap()
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
                        let (_, sender) = entry.insert((tokio::spawn(resolver), sender));

                        let unresolved_neighbours = self
                            .neighbour_store
                            .read()
                            .unwrap()
                            .unresolved
                            .values()
                            .filter(|n| n.if_index == addr.if_index)
                            .map(|n| n.mac)
                            .collect::<Vec<_>>();

                        if !unresolved_neighbours.is_empty() {
                            sender.try_send(unresolved_neighbours).unwrap();
                        }
                    }
                    Err(e) => eprintln!("Failed to create neighbour resolver: {}", e),
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
        ) {
            self.listen_addresses_notifier.send(()).unwrap();
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

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        loop {
            match self.state {
                BehaviourState::GettingBatmanAddr(ref mut behaviour) => match behaviour.poll(cx) {
                    Poll::Ready(Ok(batman_addr)) => {
                        println!("Got Batman address: {}", batman_addr);

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
                        eprintln!("Failed to get Batman address: {}", e);
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
