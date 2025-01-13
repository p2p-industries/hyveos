use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
};
use hyveos_core::grpc::{self, discovery_server::Discovery};
use hyveos_p2p_stack::Client;
use libp2p::kad::GetProvidersOk;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{kv::convert_key, ServerStream, TonicResult};

pub struct DiscoveryServer {
    client: Client,
}

impl DiscoveryServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Discovery for DiscoveryServer {
    type GetProvidersStream = ServerStream<grpc::Peer>;

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

    async fn stop_providing(
        &self,
        request: TonicRequest<grpc::DhtKey>,
    ) -> TonicResult<grpc::Empty> {
        let key = request.into_inner();

        tracing::debug!(request=?key, "Received stop_providing request");

        self.client
            .kad()
            .stop_providing(convert_key(key)?)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(TonicResponse::new(grpc::Empty {}))
    }

    async fn get_own_id(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Peer> {
        tracing::debug!("Received get_own_id request");

        Ok(TonicResponse::new(self.client.peer_id().into()))
    }
}
