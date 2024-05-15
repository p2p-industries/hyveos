use libp2p::{
    gossipsub, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, ping,
    swarm::NetworkBehaviour,
};

use super::{req_resp, round_trip};

#[cfg(feature = "location")]
use super::location;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    #[cfg(feature = "batman")]
    pub batman_neighbours: libp2p_batman_adv::Behaviour,
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub req_resp: req_resp::Behaviour,
    pub round_trip: round_trip::Behaviour,
    #[cfg(feature = "location")]
    pub location: location::Behaviour,
}

impl MyBehaviour {
    pub fn new(keypair: &Keypair) -> Self {
        let public = keypair.public();
        let peer_id = public.to_peer_id();
        Self {
            #[cfg(feature = "batman")]
            batman_neighbours: libp2p_batman_adv::Behaviour::new(
                libp2p_batman_adv::Config::default(),
                peer_id,
            ),
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
            req_resp: req_resp::new(),
            round_trip: round_trip::new(),
            #[cfg(feature = "location")]
            location: location::new(),
        }
    }
}
