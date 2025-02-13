use std::{error::Error, marker::PhantomData, time::Duration};

use futures::stream::StreamExt as _;
use libp2p::{
    identity::Keypair,
    kad::Mode,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use tokio::sync::mpsc;

#[cfg(feature = "location")]
use crate::subactors::location;
#[cfg(feature = "batman")]
use crate::subactors::{debug, neighbours};
use crate::{
    behaviour::{MyBehaviour, MyBehaviourEvent},
    client::Client,
    command::Command,
    subactors::{apps, file_transfer, gossipsub, kad, ping, req_resp, round_trip},
};

const CHANNEL_CAP: usize = 10;

pub trait SubActor: Default {
    type SubCommand: Send;
    type CommandError: Error;
    type Event;
    type EventError: Error;

    fn new(_peer_id: PeerId) -> Self {
        Default::default()
    }

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
    ReqResp,
    Apps,
    FileTransfer,
    Debug,
    EventError,
    CommandError,
> {
    swarm: Swarm<MyBehaviour>,
    receiver: mpsc::Receiver<Command>,
    kad: Kad,
    #[cfg_attr(not(feature = "mdns"), allow(dead_code))]
    mdns: Mdns,
    gossipsub: Gossipsub,
    round_trip: RoundTrip,
    #[cfg_attr(not(feature = "location"), allow(dead_code))]
    location: Location,
    ping: Ping,
    identify: Identify,
    #[cfg_attr(not(feature = "batman"), allow(dead_code))]
    neighbours: Neighbours,
    req_resp: ReqResp,
    apps: Apps,
    file_transfer: FileTransfer,
    #[cfg_attr(not(feature = "batman"), allow(dead_code))]
    debug: Debug,
    _phantom: PhantomData<EventError>,
    _command: PhantomData<CommandError>,
}

#[cfg(feature = "batman")]
pub trait DebugActor:
    SubActor<
    SubCommand = debug::Command,
    Event = <debug::Behaviour as NetworkBehaviour>::ToSwarm,
    CommandError = void::Void,
    EventError = void::Void,
>
{
}

#[cfg(feature = "batman")]
impl<T> DebugActor for T where
    T: SubActor<
        SubCommand = debug::Command,
        Event = <debug::Behaviour as NetworkBehaviour>::ToSwarm,
        CommandError = void::Void,
        EventError = void::Void,
    >
{
}

#[cfg(not(feature = "batman"))]
pub trait DebugActor:
    SubActor<
    CommandError = void::Void,
    EventError = void::Void,
    Event = void::Void,
    SubCommand = void::Void,
>
{
}

#[cfg(not(feature = "batman"))]
impl<T> DebugActor for T where
    T: SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = void::Void,
        SubCommand = void::Void,
    >
{
}

#[cfg(feature = "batman")]
pub trait NeighbourActor:
    SubActor<
    CommandError = void::Void,
    EventError = void::Void,
    Event = hyveos_libp2p_batman_adv::Event,
    SubCommand = neighbours::Command,
>
{
}

#[cfg(feature = "batman")]
impl<T> NeighbourActor for T where
    T: SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = hyveos_libp2p_batman_adv::Event,
        SubCommand = neighbours::Command,
    >
{
}

#[cfg(not(feature = "batman"))]
pub trait NeighbourActor:
    SubActor<
    CommandError = void::Void,
    EventError = void::Void,
    Event = void::Void,
    SubCommand = void::Void,
>
{
}

#[cfg(not(feature = "batman"))]
impl<T> NeighbourActor for T where
    T: SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = void::Void,
        SubCommand = void::Void,
    >
{
}

#[cfg(feature = "location")]
pub trait LocationActor:
    SubActor<
    CommandError = location::CommandError,
    EventError = location::EventError,
    Event = location::Event,
    SubCommand = location::Command,
>
{
}

#[cfg(feature = "location")]
impl<T> LocationActor for T where
    T: SubActor<
        CommandError = location::CommandError,
        EventError = location::EventError,
        Event = location::Event,
        SubCommand = location::Command,
    >
{
}

#[cfg(not(feature = "location"))]
pub trait LocationActor:
    SubActor<
    CommandError = void::Void,
    EventError = void::Void,
    Event = void::Void,
    SubCommand = void::Void,
>
{
}

#[cfg(not(feature = "location"))]
impl<T> LocationActor for T where
    T: SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = void::Void,
        SubCommand = void::Void,
    >
{
}

#[cfg(feature = "mdns")]
pub trait MdnsActor:
    SubActor<
    SubCommand = (),
    Event = libp2p::mdns::Event,
    CommandError = void::Void,
    EventError = void::Void,
>
{
}

#[cfg(not(feature = "mdns"))]
pub trait MdnsActor: SubActor<SubCommand = void::Void, Event = void::Void> {}

#[cfg(feature = "mdns")]
impl<T> MdnsActor for T where
    T: SubActor<
        SubCommand = (),
        Event = libp2p::mdns::Event,
        CommandError = void::Void,
        EventError = void::Void,
    >
{
}

#[cfg(not(feature = "mdns"))]
impl<T> MdnsActor for T where T: SubActor<SubCommand = void::Void, Event = void::Void> {}

impl<
        Kad,
        Mdns,
        Gossipsub,
        RoundTrip,
        Location,
        Ping,
        Identify,
        Neighbours,
        ReqResp,
        Apps,
        FileTransfer,
        Debug,
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
        ReqResp,
        Apps,
        FileTransfer,
        Debug,
        EventError,
        CommandError,
    >
where
    Kad: SubActor<SubCommand = kad::Command, Event = libp2p::kad::Event>,
    Mdns: MdnsActor,
    Gossipsub: SubActor<SubCommand = gossipsub::Command, Event = libp2p::gossipsub::Event>,
    RoundTrip: SubActor<
        SubCommand = round_trip::Command,
        Event = <round_trip::Behaviour as NetworkBehaviour>::ToSwarm,
        EventError = void::Void,
        CommandError = void::Void,
    >,
    Location: LocationActor,
    Ping: SubActor<
        SubCommand = ping::Command,
        Event = ping::Event,
        CommandError = void::Void,
        EventError = void::Void,
    >,
    Identify: SubActor<
        CommandError = void::Void,
        EventError = void::Void,
        Event = libp2p::identify::Event,
        SubCommand = (),
    >,
    Neighbours: NeighbourActor,
    ReqResp: SubActor<
        SubCommand = req_resp::Command,
        Event = <req_resp::Behaviour as NetworkBehaviour>::ToSwarm,
        EventError = void::Void,
        CommandError = void::Void,
    >,
    Apps: SubActor<
        SubCommand = apps::Command,
        Event = apps::Event,
        CommandError = void::Void,
        EventError = void::Void,
    >,
    FileTransfer: SubActor<
        SubCommand = file_transfer::Command,
        Event = (),
        CommandError = void::Void,
        EventError = void::Void,
    >,
    Debug: DebugActor,
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
                kad: SubActor::new(peer_id),
                mdns: SubActor::new(peer_id),
                gossipsub: SubActor::new(peer_id),
                round_trip: SubActor::new(peer_id),
                location: SubActor::new(peer_id),
                ping: SubActor::new(peer_id),
                identify: SubActor::new(peer_id),
                neighbours: SubActor::new(peer_id),
                req_resp: SubActor::new(peer_id),
                apps: SubActor::new(peer_id),
                file_transfer: SubActor::new(peer_id),
                debug: SubActor::new(peer_id),
                _phantom: PhantomData,
                _command: PhantomData,
            },
        )
    }

    pub fn setup(
        &mut self,
        listen_addrs: impl Iterator<Item = Multiaddr>,
        batman_addr: Option<Multiaddr>,
    ) {
        if let Some(batman_addr) = batman_addr {
            self.swarm.behaviour_mut().kad.with_whitelist(batman_addr);
        }
        self.swarm.behaviour_mut().kad.set_mode(Some(Mode::Server));
        for addr in listen_addrs {
            tracing::info!("Listening on: {addr:?}");
            self.swarm
                .listen_on(addr)
                .expect("Failed to listen on address");
        }
    }

    pub async fn drive(mut self) {
        loop {
            tokio::select! {
                biased;
                swarm_event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(swarm_event) {
                        tracing::error!("Error handling swarm event: {e:?}");
                    }
                },
                command_opt = self.receiver.recv() => match command_opt {
                    Some(command) => {
                        if let Err(e) = self.handle_command(command) {
                            tracing::error!("Error handling command: {e:?}");
                        }
                    },
                    None => break,
                }
            }
        }
    }

    #[tracing::instrument(skip(self), ret, err)]
    fn handle_swarm_event(
        &mut self,
        swarm_event: SwarmEvent<MyBehaviourEvent>,
    ) -> Result<(), EventError> {
        match swarm_event {
            SwarmEvent::Behaviour(MyBehaviourEvent::Kad(event)) => self
                .kad
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(Into::into),
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
                .handle_event(ping, self.swarm.behaviour_mut())
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
            SwarmEvent::Behaviour(MyBehaviourEvent::ReqResp(event)) => self
                .req_resp
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            SwarmEvent::Behaviour(MyBehaviourEvent::Apps(event)) => self
                .apps
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "mdns")]
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(event)) => self
                .mdns
                .handle_event(event, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            SwarmEvent::Behaviour(MyBehaviourEvent::FileTransfer(())) => self
                .file_transfer
                .handle_event((), self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "batman")]
            SwarmEvent::Behaviour(MyBehaviourEvent::Debug(event)) => self
                .debug
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
            Command::ReqResp(command) => self
                .req_resp
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            Command::Apps(command) => self
                .apps
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            Command::FileTransfer(command) => self
                .file_transfer
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
            #[cfg(feature = "batman")]
            Command::Debug(command) => self
                .debug
                .handle_command(command, self.swarm.behaviour_mut())
                .map_err(|e| void::unreachable(e)),
        }
    }
}
