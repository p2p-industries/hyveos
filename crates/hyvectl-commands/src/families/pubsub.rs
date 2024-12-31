use clap::Subcommand;

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

        /// Number of messages to retrieve
        #[arg(short,
            required_unless_present = "follow",
            conflicts_with = "follow")]
        n: Option<u64>,

        /// Continuously retrieve messages
        #[arg(long, short,
            required_unless_present = "n",
            conflicts_with = "n")]
        follow: bool,
    },
}

