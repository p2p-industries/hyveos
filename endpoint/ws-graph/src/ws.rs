use std::sync::Arc;

use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::Response,
};
use tokio::sync::broadcast;

use crate::{graph::ExportGraph, AppState};

async fn handle_socket(
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

pub(crate) async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let mut local_client = state.client.clone();
    if let Ok((current_graph, receiver)) = local_client.subscribe().await {
        ws.on_upgrade(|mut socket| async move {
            if let Err(err) = handle_socket(&mut socket, current_graph, receiver).await {
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
