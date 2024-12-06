use libp2p::{
    gossipsub, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    swarm::NetworkBehaviour,
};

#[cfg(feature = "batman")]
use crate::subactors::debug;
#[cfg(feature = "location")]
use crate::subactors::location;
use crate::subactors::{file_transfer, ping, req_resp, round_trip, scripting};

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    #[cfg(feature = "batman")]
    pub batman_neighbours: hyveos_libp2p_batman_adv::Behaviour,
    pub identify: identify::Behaviour,
    pub kad: hyveos_libp2p_addr_filter::Behaviour<kad::Behaviour<MemoryStore>>,
    #[cfg(feature = "mdns")]
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub req_resp: req_resp::Behaviour,
    pub ping: ping::Behaviour,
    pub round_trip: round_trip::Behaviour,
    #[cfg(feature = "location")]
    pub location: location::Behaviour,
    pub scripting: scripting::Behaviour,
    pub file_transfer: libp2p_stream::Behaviour,
    #[cfg(feature = "batman")]
    pub debug: debug::Behaviour,
}

impl MyBehaviour {
    pub fn new(keypair: &Keypair) -> Self {
        let public = keypair.public();
        let peer_id = public.to_peer_id();
        Self {
            #[cfg(feature = "batman")]
            batman_neighbours: hyveos_libp2p_batman_adv::Behaviour::new(
                hyveos_libp2p_batman_adv::Config::default(),
                peer_id,
            ),
            kad: hyveos_libp2p_addr_filter::Behaviour::new(kad::Behaviour::new(
                peer_id,
                MemoryStore::new(peer_id),
            )),
            #[cfg(feature = "mdns")]
            mdns: libp2p::mdns::tokio::Behaviour::new(
                libp2p::mdns::Config {
                    // enable_ipv6: true,
                    ..Default::default()
                },
                peer_id,
            )
            .expect("Failed to create mdns behaviour"),
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(keypair.clone()),
                gossipsub::Config::default(),
            )
            .expect("Failed to create gossipsub behaviour"),
            identify: identify::Behaviour::new(identify::Config::new(
                "/industries/id/1.0.0".into(),
                public,
            )),
            req_resp: req_resp::new(),
            ping: ping::new(),
            round_trip: round_trip::new(),
            #[cfg(feature = "location")]
            location: location::new(),
            scripting: scripting::new(),
            file_transfer: file_transfer::new(),
            #[cfg(feature = "batman")]
            debug: debug::new(),
        }
    }
}
