use batman_neighbors_core::BatmanNeighborsServerClient;
use tarpc::{client, context, tokio_serde::formats::Bincode};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::unix::connect("/var/run/batman-neighbors.sock", Bincode::default).await?;

    let client = BatmanNeighborsServerClient::new(client::Config::default(), transport).spawn();

    let neighbors = client.get_neighbors(context::current()).await?;

    match neighbors {
        Ok(neighbors) => {
            for neighbor in neighbors {
                println!("{} {} {}ms", neighbor.if_name, neighbor.mac, neighbor.last_seen_msecs);
            }
        }
        Err(e) => {
            eprintln!("Failed to get neighbors: {}", e);
        }
    }

    Ok(())
}
