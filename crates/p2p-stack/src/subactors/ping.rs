use std::{sync::Arc, time::Duration};

use libp2p::{ping, PeerId};
use tokio::sync::{broadcast, oneshot};

use crate::{
    actor::SubActor,
    behaviour::MyBehaviour,
    client::{RequestError, SpecialClient},
    impl_from_special_command,
};

#[derive(Debug)]
pub struct Actor {
    sender: broadcast::Sender<Arc<Event>>,
}

impl Default for Actor {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(10);
        Self { sender }
    }
}

#[derive(Debug)]
pub enum Command {
    Subscribe(oneshot::Sender<broadcast::Receiver<Arc<Event>>>),
}

impl_from_special_command!(Ping);

#[derive(Debug)]
pub struct Event {
    pub peer: PeerId,
    pub result: Result<Duration, String>,
}

impl From<ping::Event> for Event {
    fn from(event: ping::Event) -> Self {
        Self {
            peer: event.peer,
            result: event.result.map_err(|e| e.to_string()),
        }
    }
}

impl SubActor for Actor {
    type Event = Event;
    type SubCommand = Command;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn handle_event(
        &mut self,
        event: Self::Event,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        let _ = self.sender.send(Arc::new(event));
        Ok(())
    }

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        _behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::Subscribe(sender) => {
                let _ = sender.send(self.sender.subscribe());
            }
        }
        Ok(())
    }
}

pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn subscribe(&self) -> Result<broadcast::Receiver<Arc<Event>>, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::Subscribe(sender))
            .await
            .map_err(RequestError::Send)?;
        receiver.await.map_err(RequestError::Oneshot)
    }
}
