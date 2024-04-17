use std::borrow::Cow;

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<'a> {
    pub id: u32,
    pub peer_id: PeerId,
    pub batman_addr: Cow<'a, Multiaddr>,
    pub direct_addr: Cow<'a, Multiaddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet<'a> {
    Request(Request),
    Response(Response<'a>),
}

impl<'a> Packet<'a> {
    pub fn new_request(id: u32) -> Self {
        Packet::Request(Request { id })
    }

    pub fn new_response(
        id: u32,
        peer_id: PeerId,
        batman_addr: &'a Multiaddr,
        direct_addr: &'a Multiaddr,
    ) -> Self {
        Packet::Response(Response {
            id,
            peer_id,
            batman_addr: Cow::Borrowed(batman_addr),
            direct_addr: Cow::Borrowed(direct_addr),
        })
    }
}
