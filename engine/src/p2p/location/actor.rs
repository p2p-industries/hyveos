use crate::p2p::actor::SubActor;

#[derive(Debug, Default)]
pub struct Actor {}

#[derive(Debug, thiserror::Error)]
pub enum EventError {}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {}

impl SubActor for Actor {
    type Event = super::Event;
    type SubCommand = super::Command;
    type EventError = EventError;
    type CommandError = CommandError;
}
