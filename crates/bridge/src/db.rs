use hyveos_core::grpc::{self, db_server::Db};
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

pub struct DbServer<C> {
    client: C,
    telemetry: Telemetry,
}

impl<C: DbClient> DbServer<C> {
    pub fn new(client: C, telemetry: Telemetry) -> Self {
        Self { client, telemetry }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl<C: DbClient> Db for DbServer<C> {
    async fn put(&self, request: TonicRequest<grpc::DbRecord>) -> TonicResult<grpc::OptionalData> {
        self.telemetry.track("db.put");

        let request = request.into_inner();

        tracing::debug!(?request, "Received put request");

        let grpc::DbRecord { key, value } = request;

        let value = self
            .client
            .put(key, value)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(value.into()))
    }

    async fn get(&self, request: TonicRequest<grpc::DbKey>) -> TonicResult<grpc::OptionalData> {
        self.telemetry.track("db.get");

        let request = request.into_inner();

        tracing::debug!(?request, "Received get request");

        let grpc::DbKey { key } = request;

        let value = self
            .client
            .get(key)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(value.into()))
    }
}
