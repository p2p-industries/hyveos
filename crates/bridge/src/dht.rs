use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
};
use hyveos_core::{
    dht::Key as DhtKey,
    grpc::{self, dht_server::Dht},
};
use hyveos_p2p_stack::Client;
use libp2p::kad::{GetProvidersOk, GetRecordOk, Quorum, RecordKey};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, TonicResult};

fn convert_key(key: grpc::DhtKey) -> Result<RecordKey, Status> {
    DhtKey::from(key)
        .into_bytes()
        .map(Into::into)
        .map_err(Into::into)
}

pub struct DhtServer {
    client: Client,
}

impl DhtServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Dht for DhtServer {
    type GetProvidersStream = ServerStream<grpc::Peer>;

    async fn put_record(&self, request: TonicRequest<grpc::DhtRecord>) -> TonicResult<grpc::Empty> {
        let record = request.into_inner();

        tracing::debug!(request=?record, "Received put_record request");

        let grpc::DhtRecord { key, value } = record;

        self.client
            .kad()
            .put_record(convert_key(key)?, value.into(), None, Quorum::One)
            .await
            .map(|_| TonicResponse::new(grpc::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_record(
        &self,
        request: TonicRequest<grpc::DhtKey>,
    ) -> TonicResult<grpc::OptionalData> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received get_record request");

        let records = self
            .client
            .kad()
            .get_record(convert_key(key)?)
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

        Ok(TonicResponse::new(value.into()))
    }

    async fn provide(&self, request: TonicRequest<grpc::DhtKey>) -> TonicResult<grpc::Empty> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received provide request");

        self.client
            .kad()
            .start_providing(convert_key(key)?)
            .await
            .map(|_| TonicResponse::new(grpc::Empty {}))
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_providers(
        &self,
        request: TonicRequest<grpc::DhtKey>,
    ) -> TonicResult<Self::GetProvidersStream> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received get_providers request");

        let stream = self
            .client
            .kad()
            .get_providers(convert_key(key)?)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?
            .try_filter_map(|providers| {
                future::ready(Ok(
                    if let GetProvidersOk::FoundProviders { providers, .. } = providers {
                        Some(stream::iter(providers).map(Into::into).map(Ok))
                    } else {
                        None
                    },
                ))
            })
            .map_err(|e| Status::internal(e.to_string()))
            .try_flatten()
            .boxed();

        Ok(TonicResponse::new(stream))
    }
}
