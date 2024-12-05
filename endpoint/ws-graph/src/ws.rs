use std::sync::Arc;

use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Response,
};
use hyveos_sdk::services::debug::MessageDebugEvent;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{graph::ExportGraph, AppState};

async fn handle_mesh_topology_socket(
    socket: &mut WebSocket,
    current_graph: ExportGraph,
    mut receiver: broadcast::Receiver<Arc<ExportGraph>>,
) -> anyhow::Result<()> {
    let current_graph = serde_json::to_string(&current_graph)?;
    socket.send(Message::Text(current_graph)).await?;
    loop {
        let new_graph = receiver.recv().await?;
        let new_graph = serde_json::to_string(&new_graph)?;
        socket.send(Message::Text(new_graph)).await?;
    }
}

pub(crate) async fn mesh_topology_handler<E>(
    ws: WebSocketUpgrade,
    State(state): State<AppState<E>>,
) -> Response {
    let mut local_client = state.client;
    if let Ok((current_graph, receiver)) = local_client.subscribe().await {
        ws.on_upgrade(|mut socket| async move {
            if let Err(err) =
                handle_mesh_topology_socket(&mut socket, current_graph, receiver).await
            {
                let _ = socket
                    .send(Message::Close(Some(CloseFrame {
                        code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                        reason: err.to_string().into(),
                    })))
                    .await;
            }
        })
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to subscribe to graph updates".into())
            .unwrap()
    }
}

#[derive(Serialize)]
#[serde(tag = "success", rename_all = "camelCase")]
enum MessagesEvent<T> {
    Success(T),
    Error(String),
}

async fn handle_messages_socket<E: ToString>(
    socket: &mut WebSocket,
    mut receiver: broadcast::Receiver<Arc<Result<MessageDebugEvent, E>>>,
) -> anyhow::Result<()> {
    loop {
        let res = receiver.recv().await?;
        let event = match res.as_ref() {
            Ok(message) => MessagesEvent::Success(message),
            Err(err) => MessagesEvent::Error(err.to_string()),
        };
        let message = serde_json::to_string(&event)?;
        socket.send(Message::Text(message)).await?;
    }
}

pub(crate) async fn messages_handler<E: ToString + Send + Sync + 'static>(
    ws: WebSocketUpgrade,
    State(state): State<AppState<E>>,
) -> Response {
    ws.on_upgrade(|mut socket| async move {
        if let Err(err) = handle_messages_socket(&mut socket, state.messages_receiver).await {
            let _ = socket
                .send(Message::Close(Some(CloseFrame {
                    code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                    reason: err.to_string().into(),
                })))
                .await;
        }
    })
}
