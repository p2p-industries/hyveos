use std::net::{Ipv6Addr, SocketAddr};

use axum::{routing::get, Router};
use futures::Stream;
use p2p_industries_sdk::services::debug::MeshTopologyEvent;

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
        eprintln!("Error: {err:?}");
    });

    let state = AppState { client };
    let app = Router::new()
        .route("/ws", get(ws::handler))
        .with_state(state);
    let listener =
        tokio::net::TcpListener::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 3000))
            .await
            .unwrap();
    axum::serve(listener, app).await.unwrap();
}
