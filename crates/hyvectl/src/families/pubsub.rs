use p2p_industries_sdk::P2PConnection;
use clap::Subcommand;
use async_trait::async_trait;
use std::error::Error;
use crate::util::CommandFamily;

#[derive(Subcommand)]
pub enum PubSub {
    #[command(about = "Publish the message to the given topic")]
    Publish {
        topic: String,
        message: String
    },

    #[command(about = "Retrieve messages from the given topic")]
    Get {
        topic: String,

        #[arg(short,
            required_unless_present = "follow",
            conflicts_with = "follow",
            help = "Number of messages to retrieve")]
        n: Option<u64>,

        #[arg(long, short,
            required_unless_present = "n",
            conflicts_with = "n",
            help = "Continuously retrieve messages")]
        follow: bool,
    },
}

#[async_trait]
impl CommandFamily for PubSub {
    async fn run(self, connection: &P2PConnection) -> Result<(), Box<dyn Error>> {
        match self {
            PubSub::Publish { topic, message} => {
                let result = connection.gossipsub()
                    .publish(topic, message).await?;
                println!("Published result {result:?}");
                Ok(())
            },
            PubSub::Get {topic, n, follow} => {
                // TODO
                Ok(())
            }
        }
    }
}