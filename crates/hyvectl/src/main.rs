mod util;
mod families;
use families::{kv, pubsub};

use clap::{Parser, Subcommand};
use p2p_industries_sdk::P2PConnection;
use std::error::Error;
use util::CommandFamily;
use async_trait::async_trait;

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl", version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Families,
}

#[derive(Subcommand)]
enum Families {
    #[command(subcommand, about = "Key-Value Store")]
    KV(kv::Kv),
    #[command(subcommand, about = "Publisher-Subscriber Service")]
    PubSub(pubsub::PubSub),

}


#[async_trait]
impl CommandFamily for Families {
    async fn run(self, connection: &P2PConnection) -> Result<(), Box<dyn Error>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
            Families::PubSub(cmd) => cmd.run(connection).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let connection = P2PConnection::get().await?;

    cli.command.run(connection).await?;

    Ok(())
}
