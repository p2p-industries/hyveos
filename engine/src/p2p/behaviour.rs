use libp2p::{
    gossipsub, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, ping,
    swarm::NetworkBehaviour,
};

use super::round_trip;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub round_trip: round_trip::Behaviour,
}

impl MyBehaviour {
    pub fn new(keypair: &Keypair) -> Self {
        let public = keypair.public();
        let peer_id = public.to_peer_id();
        Self {
            ping: ping::Behaviour::default(),
            kad: kad::Behaviour::new(peer_id, MemoryStore::new(peer_id)),
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(keypair.clone()),
                gossipsub::Config::default(),
            )
            .expect("Failed to create gossipsub behaviour"),
            identify: identify::Behaviour::new(identify::Config::new(
                "/industries/id/1.0.0".into(),
                public,
            )),
            mdns: mdns::tokio::Behaviour::new(
                mdns::Config {
                    enable_ipv6: false,
                    ..Default::default()
                },
                peer_id,
            )
            .expect("Failed to init mdns"),
            round_trip: round_trip::new(),
        }
    }
}
