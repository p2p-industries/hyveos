use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u32,
}

impl Request {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u32,
    pub peer_id: PeerId,
    pub batman_addr: Multiaddr,
    pub direct_addr: Multiaddr,
}

impl Response {
    pub fn new(id: u32, peer_id: PeerId, batman_addr: Multiaddr, direct_addr: Multiaddr) -> Self {
        Self {
            id,
            peer_id,
            batman_addr,
            direct_addr,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Request(Request),
    Response(Response),
}

impl From<Request> for Packet {
    fn from(request: Request) -> Self {
        Self::Request(request)
    }
}

impl From<Response> for Packet {
    fn from(response: Response) -> Self {
        Self::Response(response)
    }
}
