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
    },
}

