mod packet;
mod socket;

use std::{
    collections::VecDeque,
    future::Future,
    io,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

use futures::SinkExt as _;
use hashlink::{linked_hash_map::Entry, LinkedHashMap};
use libp2p::{Multiaddr, PeerId};
use macaddress::MacAddress;
use socket2::{Domain, Socket, Type};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    time::Sleep,
};
use tokio_util::sync::PollSender;

use crate::{Config, ResolvedNeighbour};

use self::{packet::Packet, socket::AsyncSocket as _};

use super::if_watcher::IfAddr;

const NEIGHBOUR_RESOLUTION_PORT: u16 = 5354; // TODO: select port

pub struct NeighbourResolver {
    config: Config,
    local_peer_id: PeerId,
    batman_addr: Multiaddr,
    direct_addr: Multiaddr,
    if_addr: IfAddr,
    recv_socket: UdpSocket,
    send_socket: UdpSocket,
    recv_buffer: [u8; 4096],
    send_buffer: VecDeque<(Ipv6Addr, Vec<u8>)>,
    discovered_receiver: Receiver<Vec<MacAddress>>,
    resolve_timeouts: LinkedHashMap<u32, (MacAddress, Pin<Box<Sleep>>, u32)>,
    resolved: VecDeque<Result<ResolvedNeighbour, MacAddress>>,
    resolved_sender: PollSender<Result<ResolvedNeighbour, MacAddress>>,
}

impl NeighbourResolver {
    pub fn new(
        config: Config,
        local_peer_id: PeerId,
        batman_addr: Multiaddr,
        direct_addr: Multiaddr,
        if_addr: IfAddr,
        resolved_sender: Sender<Result<ResolvedNeighbour, MacAddress>>,
    ) -> io::Result<(Self, Sender<Vec<MacAddress>>)> {
        let recv_socket = {
            let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(socket2::Protocol::UDP))?;
            socket.set_reuse_address(true)?;
            socket.set_reuse_port(true)?;
            socket.set_nonblocking(true)?;
            socket.bind(&if_addr.with_port(NEIGHBOUR_RESOLUTION_PORT).into())?;
            UdpSocket::from_std(std::net::UdpSocket::from(socket))?
        };

        let send_socket = {
            let socket = std::net::UdpSocket::bind(if_addr.with_port(0))?;
            socket.set_nonblocking(true)?;
            UdpSocket::from_std(socket)?
        };

        let (discovered_sender, discovered_receiver) = tokio::sync::mpsc::channel(2);

        Ok((
            Self {
                config,
                local_peer_id,
                batman_addr,
                direct_addr,
                if_addr,
                recv_socket,
                send_socket,
                recv_buffer: [0; 4096],
                send_buffer: VecDeque::new(),
                discovered_receiver,
                resolve_timeouts: LinkedHashMap::new(),
                resolved: VecDeque::new(),
                resolved_sender: PollSender::new(resolved_sender),
            },
            discovered_sender,
        ))
    }

    fn send_request(&mut self, mac: MacAddress, retries: u32) {
        if retries < self.config.request_retries {
            println!("Sending request for {:?} ({} retries)", mac, retries);

            let id = rand::random();
            let sleep = Box::pin(tokio::time::sleep(self.config.request_timeout));
            let packet = Packet::new_request(id);

            let packet = bincode::serialize(&packet).expect("Failed to serialize packet");

            self.send_buffer.push_back((mac.into(), packet));

            self.resolve_timeouts.insert(id, (mac, sleep, retries));
        } else {
            println!("Failed to resolve {}", mac);
            self.resolved.push_back(Err(mac));
        }
    }
}

impl Future for NeighbourResolver {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            if let Poll::Ready(Some(macs)) = this.discovered_receiver.poll_recv(cx) {
                println!("Discovered neighbours: {:?}", macs);
                for mac in macs {
                    this.send_request(mac, 0);
                }
            }

            if let Some((addr, packet)) = this.send_buffer.pop_front() {
                println!("Sending packet to {}", addr);
                let sock_addr = SocketAddr::new(IpAddr::V6(addr), NEIGHBOUR_RESOLUTION_PORT);

                match this.send_socket.poll_write(cx, &packet, sock_addr) {
                    Poll::Ready(Ok(_)) => continue,
                    Poll::Ready(Err(err)) => {
                        eprintln!("Failed to send packet: {}", err);
                        continue;
                    }
                    Poll::Pending => {
                        this.send_buffer.push_front((addr, packet));
                    }
                }
            }

            if !this.resolved.is_empty() {
                if this.resolved_sender.poll_ready_unpin(cx).is_ready() {
                    if this
                        .resolved_sender
                        .send_item(this.resolved.pop_front().unwrap())
                        .is_err()
                    {
                        return Poll::Ready(());
                    }

                    continue;
                }
            }

            match this
                .recv_socket
                .poll_read(cx, &mut this.recv_buffer)
                .map_ok(|(len, from_addr)| {
                    bincode::deserialize::<Packet>(&this.recv_buffer[..len])
                        .map(|packet| (packet, from_addr))
                }) {
                Poll::Ready(Ok(Ok((Packet::Request(req), SocketAddr::V6(from_addr))))) => {
                    println!("Received request from {}", from_addr);
                    let packet = Packet::new_response(
                        req.id,
                        this.local_peer_id,
                        &this.batman_addr,
                        &this.direct_addr,
                    );

                    let packet = bincode::serialize(&packet).expect("Failed to serialize packet");

                    this.send_buffer.push_back((*from_addr.ip(), packet));
                    continue;
                }
                Poll::Ready(Ok(Ok((Packet::Response(res), _)))) => {
                    println!("Received response for {}", res.id);
                    if let Some((mac, _, _)) = this.resolve_timeouts.remove(&res.id) {
                        this.resolved.push_back(Ok(ResolvedNeighbour {
                            peer_id: res.peer_id,
                            if_index: this.if_addr.if_index,
                            mac,
                            direct_addr: res.direct_addr.into_owned(),
                            batman_addr: res.batman_addr.into_owned(),
                        }));
                    }
                }
                Poll::Ready(Err(err)) if err.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Poll::Ready(Err(err)) => {
                    eprintln!("Failed to receive packet: {}", err);
                    return Poll::Ready(());
                }
                Poll::Ready(Ok(Err(err))) => {
                    eprintln!("Failed to deserialize packet: {}", err);
                    continue;
                }
                Poll::Pending => {}
                _ => panic!("Unexpected IPv4 packet"),
            }

            if let Some((id, _)) = this.resolve_timeouts.front() {
                let Entry::Occupied(mut entry) = this.resolve_timeouts.entry(*id) else {
                    unreachable!();
                };

                if entry.get_mut().1.as_mut().poll(cx).is_ready() {
                    println!("Timeout for {}", entry.get().0);

                    let (mac, _, retries) = entry.remove();

                    this.send_request(mac, retries + 1);

                    continue;
                }
            }

            return Poll::Pending;
        }
    }
}
