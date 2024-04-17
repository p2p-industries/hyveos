use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u32,
    pub peer_id: PeerId,
    pub batman_addr: Multiaddr,
    pub direct_addr: Multiaddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Request(Request),
    Response(Response),
}

impl Packet {
    pub fn new_request(id: u32) -> Self {
        Packet::Request(Request { id })
    }

    pub fn new_response(
        id: u32,
        peer_id: PeerId,
        batman_addr: Multiaddr,
        direct_addr: Multiaddr,
    ) -> Self {
        Packet::Response(Response {
            id,
            peer_id,
            batman_addr,
            direct_addr,
        })
    }
}
