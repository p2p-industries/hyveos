use std::{
    net::{Ipv6Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing::get, Router};
use futures::Stream;
use hyveos_sdk::services::debug::{MeshTopologyEvent, MessageDebugEvent};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, decompression::DecompressionLayer};

#[cfg(feature = "faker")]
mod faker;

mod graph;
mod ws;

pub(crate) struct AppState<E> {
    client: graph::ParallelGraphClient,
    messages_receiver: broadcast::Receiver<Arc<Result<MessageDebugEvent, E>>>,
}

impl<E> Clone for AppState<E> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            messages_receiver: self.messages_receiver.resubscribe(),
        }
    }
}

#[cfg(not(feature = "faker"))]
async fn get_stream() -> anyhow::Result<(
    impl Stream<Item = Result<MeshTopologyEvent, hyveos_sdk::Error>>,
    broadcast::Receiver<Arc<Result<MessageDebugEvent, hyveos_sdk::Error>>>,
)> {
    use futures::stream::StreamExt as _;

    let connection = hyveos_sdk::P2PConnection::get().await?;

    let (messages_sender, messages_receiver) = broadcast::channel(10);

    tokio::spawn({
        let mut debug = connection.debug();
        async move {
            let mut messages = debug.subscribe_messages().await.unwrap();
            while let Some(event) = messages.next().await {
                let _ = messages_sender.send(Arc::new(event));
            }
        }
    });

    Ok((
        connection.debug().subscribe_mesh_topology().await?,
        messages_receiver,
    ))
}

#[cfg(feature = "faker")]
async fn get_stream() -> anyhow::Result<(
    impl Stream<Item = Result<MeshTopologyEvent, std::convert::Infallible>>,
    broadcast::Receiver<Arc<Result<MessageDebugEvent, std::convert::Infallible>>>,
)> {
    use faker::faker_oracle;
    use futures::stream::StreamExt as _;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    let (mesh_topology_sender, mesh_topology_receiver) = mpsc::channel(10);
    let (messages_sender, messages_receiver) = broadcast::channel(10);

    tokio::spawn(async move {
        let mut faker_oracle = faker_oracle();
        while let Some(event) = faker_oracle.next().await {
            match event {
                Ok(event) => {
                    let _ = mesh_topology_sender.send(Ok(event)).await;
                }
                Err(event) => {
                    let _ = messages_sender.send(Arc::new(Ok(event)));
                }
            }
        }
    });

    Ok((
        ReceiverStream::new(mesh_topology_receiver),
        messages_receiver,
    ))
}

#[tokio::main]
async fn main() {
    let (mesh_topology_stream, messages_receiver) = get_stream().await.unwrap();
    let (graph, client) = graph::ParallelGraph::new(mesh_topology_stream);
    tokio::spawn(async move {
        let err = graph.run().await.map(|_| ()).unwrap_err();
        eprintln!("Error running the graph processor: {err:?}");
    });

    let state = AppState {
        client,
        messages_receiver,
    };

    let app = Router::new()
        .route("/ws-mesh-topology", get(ws::mesh_topology_handler))
        .route("/ws-messages", get(ws::messages_handler))
        .with_state(state);
    let app = app.layer(
        ServiceBuilder::new()
            .layer(DecompressionLayer::new())
            .layer(CompressionLayer::new()),
    );

    #[cfg(feature = "frontend")]
    let app = app.nest_service(
        "/",
        tower_http::services::ServeDir::new(
            std::env::var("FRONTEND_PATH").expect("Frontend dir is not set"),
        )
        .precompressed_br()
        .precompressed_gzip(),
    );
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Invalid port was passed");
    let listener =
        tokio::net::TcpListener::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port))
            .await
            .expect("Failed to bind to port");
    println!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
