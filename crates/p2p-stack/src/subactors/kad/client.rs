use std::time::Instant;

use futures::stream::Stream;
use libp2p::kad::{
    AddProviderError, AddProviderOk, BootstrapError, BootstrapOk, GetProvidersError,
    GetProvidersOk, GetRecordError, GetRecordOk, PutRecordError, PutRecordOk, Quorum, Record,
    RecordKey,
};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use void::Void;

use super::Command;
use crate::client::{RequestError, RequestResult, SpecialClient};

#[derive(Clone)]
pub struct Client {
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}

impl Client {
    pub async fn put_record(
        &self,
        key: RecordKey,
        value: Vec<u8>,
        expires: Option<Instant>,
        quorum: Quorum,
    ) -> RequestResult<PutRecordOk, PutRecordError> {
        let record = Record {
            key,
            value,
            expires,
            publisher: Some(self.inner.peer_id),
        };
        let (sender, receiver) = oneshot::channel();
        self.inner
            .request(
                Command::PutRecord {
                    record,
                    quorum,
                    sender,
                },
                receiver,
            )
            .await
    }

    pub async fn get_record(
        &self,
        key: RecordKey,
    ) -> Result<impl Stream<Item = Result<GetRecordOk, GetRecordError>>, RequestError> {
        let (sender, receiver) = mpsc::channel(10);
        self.inner
            .send(Command::GetRecord { key, sender })
            .await
            .map_err(RequestError::Send)?;
        Ok(ReceiverStream::new(receiver))
    }

    pub async fn remove_record(&self, key: RecordKey) -> RequestResult<(), Void> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .request(Command::RemoveRecord { key, sender }, receiver)
            .await?;
        Ok(())
    }

    pub async fn bootstrap(
        &self,
    ) -> Result<impl Stream<Item = Result<BootstrapOk, BootstrapError>>, RequestError> {
        let (sender, receiver) = mpsc::channel(10);
        self.inner
            .send(Command::Bootstrap { sender })
            .await
            .map_err(RequestError::Send)?;
        Ok(ReceiverStream::new(receiver))
    }

    pub async fn get_providers(
        &self,
        key: RecordKey,
    ) -> Result<impl Stream<Item = Result<GetProvidersOk, GetProvidersError>>, RequestError> {
        let (sender, receiver) = mpsc::channel(10);
        self.inner
            .send(Command::GetProviders { key, sender })
            .await
            .map_err(RequestError::Send)?;
        Ok(ReceiverStream::new(receiver))
    }

    pub async fn start_providing(
        &self,
        key: RecordKey,
    ) -> RequestResult<AddProviderOk, AddProviderError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .request(Command::StartProviding { key, sender }, receiver)
            .await
    }

    pub async fn stop_providing(&self, key: RecordKey) -> RequestResult<(), Void> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .request(Command::StopProviding { key, sender }, receiver)
            .await?;
        Ok(())
    }
}
