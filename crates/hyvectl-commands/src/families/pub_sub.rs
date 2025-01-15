use clap::Subcommand;

#[derive(Subcommand)]
pub enum PubSub {
    /// Publish the message to the given topic
    Publish {
        /// Topic in which to publish
        topic: String,
        /// Message to publish
        message: String,
    },

    /// Retrieve messages from the given topic
    Get {
        /// Topic in which to listen for messages
        topic: String,
    },
}
