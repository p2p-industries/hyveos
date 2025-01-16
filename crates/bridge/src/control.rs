use std::sync::Arc;

use hyveos_core::grpc::{self, control_server::Control};
use hyveos_p2p_stack::Client;
use tokio::sync::Notify;
use tonic::{Request as TonicRequest, Response as TonicResponse};

use crate::{Telemetry, TonicResult};

pub struct ControlServer {
    client: Client,
    heartbeat_sender: Option<Arc<Notify>>,
    telemetry: Telemetry,
}

impl ControlServer {
    pub fn new(
        client: Client,
        heartbeat_sender: Option<Arc<Notify>>,
        telemetry: Telemetry,
    ) -> Self {
        Self {
            client,
            heartbeat_sender,
            telemetry,
        }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Control for ControlServer {
    async fn heartbeat(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Empty> {
        self.telemetry.track("control.heartbeat");
        tracing::debug!("Received heartbeat request");

        if let Some(sender) = &self.heartbeat_sender {
            sender.notify_one();
        }

        Ok(TonicResponse::new(grpc::Empty {}))
    }

    async fn get_id(&self, _request: TonicRequest<grpc::Empty>) -> TonicResult<grpc::Peer> {
        self.telemetry.track("control.get_id");
        tracing::debug!("Received get_id request");

        Ok(TonicResponse::new(self.client.peer_id().into()))
    }
}
