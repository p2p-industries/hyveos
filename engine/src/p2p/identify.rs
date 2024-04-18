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
            if (segments[0] & 0xffc0) != 0xfe80 {
                return false;
            }
            true
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;

    use libp2p::{multiaddr::Protocol, Multiaddr};
    use macaddress::{Eui48, MacAddress};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::p2p::identify::check_multiaddress_for_eui64_derived;

    // Generate ULA IPv6 address
    fn generate_ula(mut gen: impl Rng) -> Ipv6Addr {
        let mut segments = [0u16; 8];
        segments[0] = 0xfd00 | (gen.gen::<u16>() & 0xff);
        for i in segments.iter_mut().take(7).skip(1) {
            *i = gen.gen();
        }
        Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )
    }

    #[test]
    fn test_derived() {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..256 {
            let ip = Ipv6Addr::from(MacAddress::from(Eui48::new(rng.gen())));
            let addr = Multiaddr::empty()
                .with(Protocol::Ip6(ip))
                .with(Protocol::Udp(rng.gen()))
                .with(Protocol::QuicV1);
            assert!(
                check_multiaddress_for_eui64_derived(&addr),
                "addr: {addr:?}",
            );
        }

        for _ in 0..256 {
            let ip = generate_ula(&mut rng);
            let addr = Multiaddr::empty()
                .with(Protocol::Ip6(ip))
                .with(Protocol::Udp(rng.gen()))
                .with(Protocol::QuicV1);
            assert!(
                !check_multiaddress_for_eui64_derived(&addr),
                "addr: {addr:?}",
            );
        }
    }
}
