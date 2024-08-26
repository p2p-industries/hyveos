use std::net::{Ipv6Addr, SocketAddr};

use axum::{routing::get, Router};
use futures::Stream;
use p2p_industries_sdk::services::debug::MeshTopologyEvent;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, decompression::DecompressionLayer};

#[cfg(feature = "faker")]
mod faker;

mod graph;
mod ws;

#[derive(Clone)]
pub(crate) struct AppState {
    client: graph::ParallelGraphClient,
}

#[cfg(not(feature = "faker"))]
async fn get_stream(
) -> anyhow::Result<impl Stream<Item = Result<MeshTopologyEvent, p2p_industries_sdk::Error>>> {
    let connection = p2p_industries_sdk::P2PConnection::get().await?;
    Ok(connection.debug().subscribe_mesh_topology().await?)
}

#[cfg(feature = "faker")]
async fn get_stream(
) -> anyhow::Result<impl Stream<Item = Result<MeshTopologyEvent, std::convert::Infallible>>> {
    use faker::faker_oracle;
    use futures::stream::StreamExt as _;

    Ok(faker_oracle().map(Ok))
}

#[tokio::main]
async fn main() {
    let stream = get_stream().await.unwrap();
    let (graph, client) = graph::ParallelGraph::new(stream);
    tokio::spawn(async move {
        let err = graph.run().await.map(|_| ()).unwrap_err();
        eprintln!("Error running the graph processor: {err:?}");
    });

    let state = AppState { client };
    let app = Router::new()
        .route("/ws", get(ws::handler))
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
