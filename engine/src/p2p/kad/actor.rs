use std::collections::HashMap;

use libp2p::kad::{
    store, AddProviderError, AddProviderOk, BootstrapError, BootstrapOk, Event, GetProvidersError,
    GetProvidersOk, GetRecordError, GetRecordOk, NoKnownPeers, ProgressStep, PutRecordError,
    PutRecordOk, QueryId, QueryResult,
};
use tokio::sync::mpsc::error::TrySendError;
use void::Void;

use crate::p2p::{
    actor::SubActor,
    behaviour::MyBehaviour,
    command::{SendMultipleResult, SendResult},
};

use super::Command;

pub type QueryTracker<T, E> = HashMap<QueryId, SendResult<T, E>>;
pub type QueryMultipleTracker<T, E> = HashMap<QueryId, SendMultipleResult<T, E>>;

#[derive(Default)]
pub struct Actor {
    put_record_queries: QueryTracker<PutRecordOk, PutRecordError>,
    get_record_queries: QueryMultipleTracker<GetRecordOk, GetRecordError>,
    bootstrap_queries: QueryMultipleTracker<BootstrapOk, BootstrapError>,
    get_providers_queries: QueryMultipleTracker<GetProvidersOk, GetProvidersError>,
    start_providing_queries: QueryTracker<AddProviderOk, AddProviderError>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Kad Storage error: {0}")]
    Store(#[from] store::Error),
    #[error("Kad error: {0}")]
    NoKnownPeers(#[from] NoKnownPeers),
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("QueryId not found: {0}")]
    QueryIdNotFound(QueryId),
    #[error("Sending result of get record failed trying to send result: {0:?}")]
    GetRecordSendError(TrySendError<Result<GetRecordOk, GetRecordError>>),
    #[error("Sending result of put record failed trying to send result: {0:?}")]
    PutRecordSendError(Result<PutRecordOk, PutRecordError>),
    #[error("Sending result of bootstrap failed trying to send result: {0:?}")]
    BootstrapSendError(TrySendError<Result<BootstrapOk, BootstrapError>>),
    #[error("Sending result of get providers failed trying to send result: {0:?}")]
    GetProvidersSendError(TrySendError<Result<GetProvidersOk, GetProvidersError>>),
    #[error("Sending result of start providing failed trying to send result: {0:?}")]
    AddProviderSendError(Result<AddProviderOk, AddProviderError>),
}

macro_rules! call_behaviour {
    (throws; $self:ident, $fn:ident, $map:ident, $behaviour:ident, $sender:ident; $($args:ident),*) => {{
        let query_id = $behaviour.kad.$fn( $($args,)* )?;
        // We can ignore that here becauase QueryId is unique
        let _ = $self.$map.insert(query_id, $sender);
        Ok(())
    }};
    ($self:ident, $fn:ident, $map:ident, $behaviour:ident, $sender:ident; $($args:ident),*) => {{
        let query_id = $behaviour.kad.$fn( $($args,)* );
        // We can ignore that here becauase QueryId is unique
        let _ = $self.$map.insert(query_id, $sender);
        Ok(())
    }};
}

impl SubActor for Actor {
    type SubCommand = Command;
    type CommandError = CommandError;
    type Event = Event;
    type EventError = EventError;

    fn handle_event(
        &mut self,
        event: Self::Event,
        _: &mut MyBehaviour,
    ) -> Result<(), Self::EventError> {
        match event {
            Event::OutboundQueryProgressed {
                id, result, step, ..
            } => self.handle_outbound_query_progressed(id, result, step),
            _ => Ok(()),
        }
    }

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::PutRecord {
                record,
                quorum,
                sender,
            } => {
                call_behaviour!(
                    throws;
                    self,
                    put_record,
                    put_record_queries,
                    behaviour,
                    sender;
                    record,
                    quorum
                )
            }
            Command::GetRecord { key, sender } => {
                call_behaviour!(self, get_record, get_record_queries, behaviour, sender; key)
            }
            Command::Bootstrap { sender } => call_behaviour!(
                throws;
                self,
                bootstrap,
                bootstrap_queries,
                behaviour,
                sender;
            ),
            Command::GetProviders { key, sender } => {
                call_behaviour!(self, get_providers, get_providers_queries, behaviour, sender; key)
            }
            Command::StartProviding { key, sender } => {
                call_behaviour!(
                    throws;
                    self,
                    start_providing,
                    start_providing_queries,
                    behaviour,
                    sender;
                    key
                )
            }
        }
    }
}

impl Actor {
    pub fn handle_outbound_query_progressed(
        &mut self,
        id: QueryId,
        result: QueryResult,
        step: ProgressStep,
    ) -> Result<(), EventError> {
        match result {
            QueryResult::GetRecord(res) => {
                let sender = self
                    .get_record_queries
                    .remove(&id)
                    .ok_or(EventError::QueryIdNotFound(id))?;
                sender
                    .try_send(res)
                    .map_err(EventError::GetRecordSendError)?;
                if !step.last {
                    self.get_record_queries.insert(id, sender);
                }
                Ok(())
            }
            QueryResult::PutRecord(res) => {
                let sender = self
                    .put_record_queries
                    .remove(&id)
                    .ok_or(EventError::QueryIdNotFound(id))?;
                sender.send(res).map_err(EventError::PutRecordSendError)?;
                Ok(())
            }
            QueryResult::Bootstrap(res) => {
                let sender = self
                    .bootstrap_queries
                    .remove(&id)
                    .ok_or(EventError::QueryIdNotFound(id))?;
                sender
                    .try_send(res)
                    .map_err(EventError::BootstrapSendError)?;
                if !step.last {
                    self.bootstrap_queries.insert(id, sender);
                }
                Ok(())
            }
            QueryResult::GetProviders(res) => {
                let sender = self
                    .get_providers_queries
                    .remove(&id)
                    .ok_or(EventError::QueryIdNotFound(id))?;
                sender
                    .try_send(res)
                    .map_err(EventError::GetProvidersSendError)?;
                if !step.last {
                    self.get_providers_queries.insert(id, sender);
                }
                Ok(())
            }
            QueryResult::StartProviding(res) => {
                let sender = self
                    .start_providing_queries
                    .remove(&id)
                    .ok_or(EventError::QueryIdNotFound(id))?;
                sender.send(res).map_err(EventError::AddProviderSendError)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
