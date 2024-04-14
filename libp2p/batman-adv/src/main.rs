use batman_neighbours_core::BatmanNeighboursServerClient;
use tarpc::{client, context, tokio_serde::formats::Bincode};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::unix::connect("/var/run/batman-neighbours.sock", Bincode::default).await?;

    let client = BatmanNeighboursServerClient::new(client::Config::default(), transport).spawn();

    let neighbours = client.get_neighbours(context::current()).await?;

    match neighbours {
        Ok(neighbours) => {
            for neighbour in neighbours {
                println!("{} {} {}ms", neighbour.if_name, neighbour.mac, neighbour.last_seen_msecs);
            }
        }
        Err(e) => {
            eprintln!("Failed to get neighbours: {}", e);
        }
    }

    Ok(())
}
