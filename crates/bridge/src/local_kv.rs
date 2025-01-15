use hyveos_core::grpc::{self, local_kv_server::LocalKv};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{Telemetry, TonicResult};

#[trait_variant::make(Send)]
pub trait DbClient: Sync + 'static {
    type Error: ToString;

    async fn put(
        &self,
        key: String,
        value: impl Into<Vec<u8>> + Send,
    ) -> Result<Option<Vec<u8>>, Self::Error>;

    async fn get(&self, key: String) -> Result<Option<Vec<u8>>, Self::Error>;
}

pub struct LocalKvServer<C> {
    client: C,
    telemetry: Telemetry,
}

impl<C: DbClient> LocalKvServer<C> {
    pub fn new(client: C, telemetry: Telemetry) -> Self {
        Self { client, telemetry }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl<C: DbClient> LocalKv for LocalKvServer<C> {
    async fn put(
        &self,
        request: TonicRequest<grpc::LocalKvRecord>,
    ) -> TonicResult<grpc::OptionalData> {
        self.telemetry.track("local_kv.put");
        let request = request.into_inner();

        tracing::debug!(?request, "Received put request");

        let grpc::LocalKvRecord { key, value } = request;

        let value = self
            .client
            .put(key, value)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(value.into()))
    }

    async fn get(
        &self,
        request: TonicRequest<grpc::LocalKvKey>,
    ) -> TonicResult<grpc::OptionalData> {
        self.telemetry.track("local_kv.get");
        let request = request.into_inner();

        tracing::debug!(?request, "Received get request");

        let grpc::LocalKvKey { key } = request;

        let value = self
            .client
            .get(key)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(value.into()))
    }
}
