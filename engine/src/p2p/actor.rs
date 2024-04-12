use std::{
    error::Error,
    marker::PhantomData,
    time::Duration,
};

use libp2p::{
    futures::StreamExt, identity::Keypair, kad::Mode, swarm::SwarmEvent, Swarm, SwarmBuilder,
};
use tokio::sync::mpsc;

use crate::p2p::behaviour::MyBehaviour;

use super::{behaviour::MyBehaviourEvent, client::Client, command::Command, gossipsub, kad};

const CHANNEL_CAP: usize = 10;

pub trait SubActor {
    type SubCommand;
    type CommandError: Error;
    type Event;
    type EventError: Error;

    fn handle_command(
        &mut self,
        _command: Self::SubCommand,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        Ok(())
    }

    fn handle_event(
        &mut self,
        _event: Self::Event,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        Ok(())
    }
}

pub struct Actor<Kad, Mdns, Gossipsub, EventError, CommandError> {
    swarm: Swarm<MyBehaviour>,
    receiver: mpsc::Receiver<Command>,
    kad: Kad,
    mdns: Mdns,
    gossipsub: Gossipsub,
    _phantom: PhantomData<EventError>,
    _command: PhantomData<CommandError>,
}

impl<Kad, Mdns, Gossipsub, EventError, CommandError>
    Actor<Kad, Mdns, Gossipsub, EventError, CommandError>
where
    Kad: SubActor<SubCommand = kad::Command, Event = libp2p::kad::Event> + Default,
    Mdns: SubActor<
            SubCommand = (),
            Event = libp2p::mdns::Event,
            EventError = void::Void,
            CommandError = void::Void,
        > + Default,
    Gossipsub:
        SubActor<SubCommand = gossipsub::Command, Event = libp2p::gossipsub::Event> + Default,
    EventError:
        Error + From<<Kad as SubActor>::EventError> + From<<Gossipsub as SubActor>::EventError>,
    CommandError:
        Error + From<<Kad as SubActor>::CommandError> + From<<Gossipsub as SubActor>::CommandError>,
{
    pub fn build(keypair: Keypair) -> (Client, Self) {
        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic()
            .with_behaviour(MyBehaviour::new)
            .expect("Failed to build swarm")
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        let peer_id = *swarm.local_peer_id();
        let (sender, receiver) = mpsc::channel(CHANNEL_CAP);
        (
            Client::new(sender, peer_id),
            Self {
                swarm,
                receiver,
                kad: Default::default(),
                mdns: Default::default(),
                gossipsub: Default::default(),
                _phantom: PhantomData,
                _command: PhantomData,
            },
        )
    }

    pub fn setup(&mut self) {
        self.swarm.behaviour_mut().kad.set_mode(Some(Mode::Server));
        self.swarm
            .listen_on("/ip6/::/udp/0/quic-v1".parse().unwrap())
            .expect("Failed to listen on IPv6");
        self.swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
            .expect("Failed to listen on IPv4");
    }

    pub async fn drive(mut self) {
        loop {
            tokio::select! {
                biased;
                swarm_event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(swarm_event) {
                        eprintln!("Error handling swarm event: {e:?}");
                    }
                },
                command_opt = self.receiver.recv() => match command_opt {
                    Some(command) => {
                        if let Err(e) = self.handle_command(command) {
                            eprintln!("Error handling command: {e:?}");
                        }
                    },
                    None => break,
                }
            }
        }
    }

    fn handle_swarm_event(
        &mut self,
        swarm_event: SwarmEvent<MyBehaviourEvent>,
    ) -> Result<(), EventError> {
        match swarm_event {
            SwarmEvent::Behaviour(MyBehaviourEvent::Kad(event)) => self
                .kad
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(Into::into),
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(event)) => self
                .mdns
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(event)) => self
                .gossipsub
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(Into::into),
            e => {
                // println!("Unhandled swarm event: {:?}", e);
                Ok(())
            }
        }
    }

    fn handle_command(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::Kad(command) => self
                .kad
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(Into::into),
            Command::Gossipsub(command) => self
                .gossipsub
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(Into::into),
        }
    }
}
