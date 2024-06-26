use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
};
use libp2p::kad::{GetProvidersOk, GetRecordOk, Quorum, RecordKey};
use p2p_stack::Client;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{
    script::{self, dht_server::Dht},
    ServerStream, TonicResult,
};

impl TryFrom<script::DhtKey> for RecordKey {
    type Error = Status;

    fn try_from(key: script::DhtKey) -> Result<Self, Status> {
        let script::DhtKey {
            topic: script::Topic { topic },
            key,
        } = key;

        let topic_bytes = topic.into_bytes();

        if topic_bytes.contains(&b'/') {
            return Err(Status::invalid_argument("Topic cannot contain '/'"));
        }

        let key_bytes = topic_bytes
            .into_iter()
            .chain(Some(b'/'))
            .chain(key)
            .collect::<Vec<_>>();

        Ok(RecordKey::new(&key_bytes))
    }
}

pub struct DhtServer {
    client: Client,
}

impl DhtServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl Dht for DhtServer {
    type GetProvidersStream = ServerStream<script::Peer>;

    async fn put_record(
        &self,
        request: TonicRequest<script::DhtPutRecord>,
    ) -> TonicResult<script::Empty> {
        let record = request.into_inner();

        tracing::debug!(request=?record, "Received put_record request");

        let script::DhtPutRecord { key, value } = record;

        self.client
            .kad()
            .put_record(key.try_into()?, value, None, Quorum::One)
            .await
            .map(|_| TonicResponse::new(script::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_record(
        &self,
        request: TonicRequest<script::DhtKey>,
    ) -> TonicResult<script::DhtGetRecord> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received get_record request");

        let records = self
            .client
            .kad()
            .get_record(key.clone().try_into()?)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let value = records
            .try_filter_map(|get_record| {
                future::ready(Ok(if let GetRecordOk::FoundRecord(record) = get_record {
                    Some(record.record.value)
                } else {
                    None
                }))
            })
            .next()
            .await
            .transpose()
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(script::DhtGetRecord { key, value }))
    }

    async fn provide(&self, request: TonicRequest<script::DhtKey>) -> TonicResult<script::Empty> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received provide request");

        self.client
            .kad()
            .start_providing(key.try_into()?)
            .await
            .map(|_| TonicResponse::new(script::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_providers(
        &self,
        request: TonicRequest<script::DhtKey>,
    ) -> TonicResult<Self::GetProvidersStream> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received get_providers request");

        let stream = self
            .client
            .kad()
            .get_providers(key.try_into()?)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?
            .try_filter_map(|providers| {
                future::ready(Ok(
                    if let GetProvidersOk::FoundProviders { providers, .. } = providers {
                        Some(stream::iter(providers).map(|peer_id| {
                            Ok(script::Peer {
                                peer_id: peer_id.to_string(),
                            })
                        }))
                    } else {
                        None
                    },
                ))
            })
            .map_err(|e| Status::internal(e.to_string()))
            .try_flatten();

        Ok(TonicResponse::new(Box::pin(stream)))
    }
}
