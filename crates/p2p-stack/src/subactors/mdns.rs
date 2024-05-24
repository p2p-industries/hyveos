use libp2p::mdns;

use crate::{actor::SubActor, behaviour::MyBehaviour};

#[derive(Debug, Default)]
pub struct Actor;

impl SubActor for Actor {
    type CommandError = void::Void;
    type Event = mdns::Event;
    type SubCommand = ();
    type EventError = void::Void;

    fn handle_command(
        &mut self,
        _command: Self::SubCommand,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Self::Event,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            mdns::Event::Discovered(peers) => peers.into_iter().for_each(|(peer_id, addr)| {
                behaviour.kad.add_address(&peer_id, addr);
                behaviour.gossipsub.add_explicit_peer(&peer_id);
            }),
            mdns::Event::Expired(peers) => peers.into_iter().for_each(|(peer_id, _)| {
                behaviour.gossipsub.remove_explicit_peer(&peer_id);
            }),
        }
        Ok(())
    }
}
