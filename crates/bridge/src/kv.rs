use futures::{
    future,
    stream::{StreamExt as _, TryStreamExt as _},
};
use hyveos_core::{
    dht::Key as DhtKey,
    grpc::{self, kv_server::Kv},
};
use hyveos_p2p_stack::Client;
use libp2p::kad::{GetRecordOk, Quorum, RecordKey};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{Telemetry, TonicResult};

pub(crate) fn convert_key(key: grpc::DhtKey) -> Result<RecordKey, Status> {
    DhtKey::from(key)
        .into_bytes()
        .map(Into::into)
        .map_err(Into::into)
}

pub struct KvServer {
    client: Client,
    telemetry: Telemetry,
}

impl KvServer {
    pub fn new(client: Client, telemetry: Telemetry) -> Self {
        Self { client, telemetry }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Kv for KvServer {
    async fn put_record(&self, request: TonicRequest<grpc::DhtRecord>) -> TonicResult<grpc::Empty> {
        self.telemetry.track("kv.put_record");

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
        self.telemetry.track("kv.get_record");

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

    async fn remove_record(&self, request: TonicRequest<grpc::DhtKey>) -> TonicResult<grpc::Empty> {
        self.telemetry.track("kv.remove_record");

        let key = request.into_inner();

        tracing::debug!(request=?key, "Received remove_record request");

        self.client
            .kad()
            .remove_record(convert_key(key)?)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(grpc::Empty {}))
    }
}
