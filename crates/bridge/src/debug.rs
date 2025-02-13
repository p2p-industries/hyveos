use drop_stream::DropStream;
use futures::{StreamExt as _, TryStreamExt as _};
use hyveos_core::grpc::{self, debug_server::Debug};
use hyveos_p2p_stack::DebugClientCommand;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, Telemetry, TonicResult};

pub struct DebugServer {
    command_sender: mpsc::Sender<DebugClientCommand>,
    telemetry: Telemetry,
}

impl DebugServer {
    pub fn new(command_sender: mpsc::Sender<DebugClientCommand>, telemetry: Telemetry) -> Self {
        Self {
            command_sender,
            telemetry,
        }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl Debug for DebugServer {
    type SubscribeMeshTopologyStream = ServerStream<grpc::MeshTopologyEvent>;
    type SubscribeMessagesStream = ServerStream<grpc::MessageDebugEvent>;

    async fn subscribe_mesh_topology(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeMeshTopologyStream> {
        self.telemetry.track("debug.subscribe_mesh_topology");
        tracing::debug!("Received subscribe_mesh_topology request");

        let command_sender = self.command_sender.clone();

        let (sender, receiver) = oneshot::channel();

        command_sender
            .send(DebugClientCommand::SubscribeNeighbourEvents(sender))
            .await
            .map_err(|_| Status::internal("Failed to send command"))?;

        let receiver = receiver
            .await
            .map_err(|_| Status::internal("Failed to receive response"))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(Into::into)
            .map_err(|e| Status::internal(e.to_string()));

        let drop_stream = DropStream::new(stream, move || {
            tokio::spawn(async move {
                let _ = command_sender
                    .send(DebugClientCommand::UnsubscribeNeighbourEvents)
                    .await;
            });
        })
        .boxed();

        Ok(TonicResponse::new(drop_stream))
    }

    async fn subscribe_messages(
        &self,
        _request: TonicRequest<grpc::Empty>,
    ) -> TonicResult<Self::SubscribeMessagesStream> {
        self.telemetry.track("debug.subscribe_messages");
        tracing::debug!("Received subscribe_messages request");

        let command_sender = self.command_sender.clone();

        let (sender, receiver) = oneshot::channel();

        command_sender
            .send(DebugClientCommand::SubscribeMessageEvents(sender))
            .await
            .map_err(|_| Status::internal("Failed to send command"))?;

        let receiver = receiver
            .await
            .map_err(|_| Status::internal("Failed to receive response"))?;

        let stream = BroadcastStream::new(receiver)
            .map_ok(Into::into)
            .map_err(|e| Status::internal(e.to_string()));

        let drop_stream = DropStream::new(stream, move || {
            tokio::spawn(async move {
                let _ = command_sender
                    .send(DebugClientCommand::UnsubscribeMessageEvents)
                    .await;
            });
        })
        .boxed();

        Ok(TonicResponse::new(drop_stream))
    }
}
