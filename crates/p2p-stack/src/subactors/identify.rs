use libp2p::{identify::Event, multiaddr::Protocol};

use crate::{actor::SubActor, behaviour::MyBehaviour};

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
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        if let Event::Received { peer_id, info } = event {
            for address in info
                .listen_addrs
                .into_iter()
                .chain(std::iter::once(info.observed_addr))
            {
                if address
                    .iter()
                    .any(|protocol| matches!(protocol, Protocol::Onion(_, _)))
                {
                    continue;
                }
                behaviour.kad.add_address(&peer_id, address);
            }
        }
        Ok(())
    }
}
