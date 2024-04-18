use libp2p::{identify::Event, multiaddr::Protocol, Multiaddr};

use super::actor::SubActor;

#[derive(Debug, Clone, Copy, Default)]
pub struct Actor;

impl SubActor for Actor {
    type SubCommand = ();
    type EventError = void::Void;
    type CommandError = void::Void;
    type Event = Event;

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut super::behaviour::MyBehaviour,
    ) -> Result<(), Self::EventError> {
        if let Event::Received { peer_id, info } = event {
            for address in info
                .listen_addrs
                .into_iter()
                .chain(std::iter::once(info.observed_addr))
            {
                if check_multiaddress_for_eui64_derived(&address) {
                    continue;
                }
                behaviour.kad.add_address(&peer_id, address);
            }
        }
        Ok(())
    }
}

fn check_multiaddress_for_eui64_derived(multiaddr: &Multiaddr) -> bool {
    multiaddr.iter().any(|addr| match addr {
        Protocol::Ip6(ip) => {
            let segments = ip.segments();
            (segments[4] == 0xfffe) && ((segments[3] & 0x0200) == 0)
        }
        _ => false,
    })
}
