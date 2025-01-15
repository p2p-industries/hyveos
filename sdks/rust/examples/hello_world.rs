use std::time::Duration;

use futures::StreamExt;
use hyveos_core::pub_sub::ReceivedMessage;
use hyveos_sdk::services::{NeighbourEvent, NeighboursService};

async fn wait_for_peers(mut neighbours: NeighboursService) -> anyhow::Result<()> {
    while let Some(event) = neighbours.subscribe().await?.next().await {
        match event? {
            NeighbourEvent::Init(peers) => {
                println!("Initial peers: {peers:?}");
                break;
            }
            NeighbourEvent::Discovered(peer) => {
                println!("Peer discovered: {peer:?}");
                break;
            }
            NeighbourEvent::Lost(peer) => {
                println!("Peer lost: {peer:?}");
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = hyveos_sdk::Connection::new().await?;

    let own_id = connection.get_id().await?;

    println!("Node {own_id} has started.");

    wait_for_peers(connection.neighbours()).await?;

    let topic = "greetings";

    let mut stream = connection.pub_sub().subscribe(topic).await?;

    tokio::time::sleep(Duration::from_secs(10)).await;

    let message = format!("Hello from {own_id}");

    connection.pub_sub().publish(topic, message).await?;

    while let Some(message) = stream.next().await {
        let ReceivedMessage {
            message, source, ..
        } = message?;
        if let Some(source) = source {
            println!("Received message from {source}: {message:?}");
        } else {
            println!("Received message from unknown source: {message:?}");
        }
    }

    Ok(())
}
