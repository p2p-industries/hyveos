use std::{
    collections::{HashSet, VecDeque},
    future::Future,
    io,
    net::IpAddr,
    pin::Pin,
    task::{ready, Context, Poll},
};

use futures::{stream::FusedStream, Stream, StreamExt as _, TryStreamExt as _};
use ifaddr::IfAddr;
use netlink_packet_core::NetlinkPayload;
use netlink_packet_route::{
    address::{AddressAttribute, AddressMessage},
    RouteNetlinkMessage,
};
use netlink_proto::{
    sys::{AsyncSocket as _, SocketAddr, TokioSocket},
    Connection,
};
use rtnetlink::constants::RTMGRP_IPV6_IFADDR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IfEvent {
    Up(IfAddr),
    Down(IfAddr),
}

pub struct IfWatcher {
    conn: Connection<RouteNetlinkMessage, TokioSocket>,
    messages: Pin<Box<dyn Stream<Item = io::Result<RouteNetlinkMessage>> + Send>>,
    addrs: HashSet<IfAddr>,
    queue: VecDeque<IfEvent>,
}

impl std::fmt::Debug for IfWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("IfWatcher")
            .field("addrs", &self.addrs)
            .finish_non_exhaustive()
    }
}

impl IfWatcher {
    pub fn new() -> io::Result<Self> {
        let (mut conn, handle, messages) = rtnetlink::new_connection()?;
        let groups = RTMGRP_IPV6_IFADDR;
        let addr = SocketAddr::new(0, groups);
        conn.socket_mut().socket_mut().bind(&addr)?;
        let get_addrs_stream = handle
            .address()
            .get()
            .execute()
            .map_ok(RouteNetlinkMessage::NewAddress)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let msg_stream = messages.filter_map(|(msg, _)| async {
            match msg.payload {
                NetlinkPayload::Error(err) => Some(Err(err.to_io())),
                NetlinkPayload::InnerMessage(msg) => Some(Ok(msg)),
                _ => None,
            }
        });
        let messages = get_addrs_stream.chain(msg_stream).boxed();
        let addrs = HashSet::default();
        let queue = VecDeque::default();
        Ok(Self {
            conn,
            messages,
            addrs,
            queue,
        })
    }

    pub fn poll_if_event(&mut self, cx: &mut Context) -> Poll<io::Result<IfEvent>> {
        loop {
            if let Some(event) = self.queue.pop_front() {
                return Poll::Ready(Ok(event));
            }
            if Pin::new(&mut self.conn).poll(cx).is_ready() {
                return Poll::Ready(Err(Self::socket_err()));
            }
            let message =
                ready!(self.messages.poll_next_unpin(cx)).ok_or_else(Self::socket_err)??;
            match message {
                RouteNetlinkMessage::NewAddress(msg) => self.add_address(msg),
                RouteNetlinkMessage::DelAddress(msg) => self.rem_address(msg),
                _ => {}
            }
        }
    }

    fn add_address(&mut self, msg: AddressMessage) {
        for addr in Self::iter_addrs(msg) {
            if self.addrs.insert(addr) {
                self.queue.push_back(IfEvent::Up(addr));
            }
        }
    }

    fn rem_address(&mut self, msg: AddressMessage) {
        for addr in Self::iter_addrs(msg) {
            if self.addrs.remove(&addr) {
                self.queue.push_back(IfEvent::Down(addr));
            }
        }
    }

    fn socket_err() -> io::Error {
        io::Error::new(io::ErrorKind::BrokenPipe, "rtnetlink socket closed")
    }

    fn iter_addrs(msg: AddressMessage) -> impl Iterator<Item = IfAddr> {
        let if_index = msg.header.index;
        msg.attributes
            .into_iter()
            .filter_map(move |attr| match attr {
                AddressAttribute::Address(IpAddr::V6(addr)) => Some(IfAddr { if_index, addr }),
                _ => None,
            })
    }
}

impl Stream for IfWatcher {
    type Item = io::Result<IfEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_if_event(cx).map(Some)
    }
}

impl FusedStream for IfWatcher {
    fn is_terminated(&self) -> bool {
        false
    }
}
