use std::{error::Error, marker::PhantomData, time::Duration};

use libp2p::{
    futures::StreamExt,
    identity::Keypair,
    kad::Mode,
    swarm::{NetworkBehaviour, SwarmEvent},
    Swarm, SwarmBuilder,
};
use tokio::sync::mpsc;

use crate::p2p::behaviour::MyBehaviour;

use super::{
    behaviour::MyBehaviourEvent, client::Client, command::Command, gossipsub, kad, ping, round_trip,
};

#[cfg(feature = "batman")]
use super::neighbours;

#[cfg(feature = "location")]
use super::location;

const CHANNEL_CAP: usize = 10;

pub trait SubActor {
    type SubCommand: Send;
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

impl SubActor for () {
    type SubCommand = void::Void;
    type CommandError = void::Void;
    type Event = void::Void;
    type EventError = void::Void;
}

pub struct Actor<
    Kad,
    Mdns,
    Gossipsub,
    RoundTrip,
    Location,
    Ping,
    Identify,
    Neighbours,
    EventError,
    CommandError,
> {
    swarm: Swarm<MyBehaviour>,
    receiver: mpsc::Receiver<Command>,
    kad: Kad,
    mdns: Mdns,
    gossipsub: Gossipsub,
    round_trip: RoundTrip,
    #[cfg_attr(not(feature = "location"), allow(dead_code))]
    location: Location,
    ping: Ping,
    identify: Identify,
    #[cfg_attr(not(feature = "batman"), allow(dead_code))]
    neighbours: Neighbours,
    _phantom: PhantomData<EventError>,
    _command: PhantomData<CommandError>,
}

#[cfg(feature = "batman")]
pub(crate) trait NeighbourActor:
    SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = libp2p_batman_adv::Event,
        SubCommand = neighbours::Command,
    > + Default
{
}

#[cfg(feature = "batman")]
impl<T> NeighbourActor for T where
    T: SubActor<
            CommandError = void::Void,
            EventError = void::Void,
            Event = libp2p_batman_adv::Event,
            SubCommand = neighbours::Command,
        > + Default
{
}

#[cfg(not(feature = "batman"))]
pub(crate) trait NeighbourActor:
    SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = void::Void,
        SubCommand = void::Void,
    > + Default
{
}

#[cfg(not(feature = "batman"))]
impl<T> NeighbourActor for T where
    T: SubActor<
            CommandError = void::Void,
            EventError = void::Void,
            Event = void::Void,
            SubCommand = void::Void,
        > + Default
{
}

#[cfg(feature = "location")]
pub(crate) trait LocationActor:
    SubActor<
        CommandError = location::CommandError,
        EventError = location::EventError,
        Event = location::Event,
        SubCommand = location::Command,
    > + Default
{
}

#[cfg(feature = "location")]
impl<T> LocationActor for T where
    T: SubActor<
            CommandError = location::CommandError,
            EventError = location::EventError,
            Event = location::Event,
            SubCommand = location::Command,
        > + Default
{
}

#[cfg(not(feature = "location"))]
pub(crate) trait LocationActor:
    SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = void::Void,
        SubCommand = void::Void,
    > + Default
{
}

#[cfg(not(feature = "location"))]
impl<T> LocationActor for T where
    T: SubActor<
            CommandError = void::Void,
            EventError = void::Void,
            Event = void::Void,
            SubCommand = void::Void,
        > + Default
{
}

impl<
        Kad,
        Mdns,
        Gossipsub,
        RoundTrip,
        Location,
        Ping,
        Identify,
        Neighbours,
        EventError,
        CommandError,
    >
    Actor<
        Kad,
        Mdns,
        Gossipsub,
        RoundTrip,
        Location,
        Ping,
        Identify,
        Neighbours,
        EventError,
        CommandError,
    >
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
    RoundTrip: SubActor<
            SubCommand = round_trip::Command,
            Event = <round_trip::Behaviour as NetworkBehaviour>::ToSwarm,
            EventError = void::Void,
            CommandError = void::Void,
        > + Default,
    Location: LocationActor,
    Ping: SubActor<
            SubCommand = ping::Command,
            Event = ping::Event,
            CommandError = void::Void,
            EventError = void::Void,
        > + Default,
    Identify: SubActor<
            CommandError = void::Void,
            EventError = void::Void,
            Event = libp2p::identify::Event,
            SubCommand = (),
        > + Default,
    Neighbours: NeighbourActor,
    EventError: Error
        + From<<Kad as SubActor>::EventError>
        + From<<Gossipsub as SubActor>::EventError>
        + From<<Location as SubActor>::EventError>,
    CommandError: Error
        + From<<Kad as SubActor>::CommandError>
        + From<<Gossipsub as SubActor>::CommandError>
        + From<<Location as SubActor>::CommandError>,
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
                round_trip: Default::default(),
                location: Default::default(),
                ping: Default::default(),
                identify: Default::default(),
                neighbours: Default::default(),
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
            SwarmEvent::Behaviour(MyBehaviourEvent::RoundTrip(event)) => self
                .round_trip
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "location")]
            SwarmEvent::Behaviour(MyBehaviourEvent::Location(event)) => self
                .location
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(Into::into),
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(ping)) => self
                .ping
                .handle_event(ping.into(), self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            SwarmEvent::Behaviour(MyBehaviourEvent::Identify(event)) => self
                .identify
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "batman")]
            SwarmEvent::Behaviour(MyBehaviourEvent::BatmanNeighbours(event)) => self
                .neighbours
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            _ => Ok(()),
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
            Command::RoundTrip(command) => self
                .round_trip
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "location")]
            Command::Location(command) => self
                .location
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(Into::into),
            Command::Ping(command) => self
                .ping
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "batman")]
            Command::Neighbours(command) => self
                .neighbours
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
        }
    }
}
