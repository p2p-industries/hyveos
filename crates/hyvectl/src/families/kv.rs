use p2p_industries_sdk::P2PConnection;
use clap::Subcommand;
use async_trait::async_trait;
use std::error::Error;
use crate::util::CommandFamily;
use futures::TryStreamExt as _;

#[derive(Subcommand)]
pub enum Kv {
    #[command(about = "Publish the value for the given key")]
    Put {
        key: String,
        value: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the value for the given key")]
    Get {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Announce that this node can provide the value for the given key")]
    Provide {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the providers for the given key")]
    GetProviders {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
}

#[async_trait]
impl CommandFamily for Kv {
    async fn run(self, connection: &P2PConnection) -> Result<(), Box<dyn Error>> {
        match self {
            Kv::Put { key, value, topic } => {
                let topic = topic.unwrap_or_default();
                let result = connection.dht()
                    .put_record(key, value, topic).await?;
                println!("Put result: {result:?}");
                Ok(())
            },
            Kv::Get { key, topic } => {
                let topic = topic.unwrap_or_default();
                let result = connection.dht()
                    .get_record(key, topic).await?;
                println!("Get result: {result:?}");
                Ok(())
            },
            Kv::Provide {key, topic} => {
                let result = connection.dht()
                    .provide(key, topic.unwrap_or_default()).await?;
                println!("Provide result: {result:?}");
                Ok(())
            },
            Kv::GetProviders {key, topic} => {
                let mut result = connection.dht()
                    .get_providers(key, topic.unwrap_or_default()).await?;

                print!("GetProviders result:");
                while let Some(peer_id) = result.try_next().await? {
                    println!("{peer_id}");
                };

                Ok(())
            },
        }
    }
}
