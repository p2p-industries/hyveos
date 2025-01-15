use clap::Subcommand;

#[derive(Subcommand)]
pub enum Kv {
    /// Publish the value for the given key
    Put {
        /// Key to publish
        key: String,
        /// Value to publish
        value: String,
        /// Topic under which to publish key
        #[arg(long)]
        topic: Option<String>,
    },
    /// Get the value for the given key
    Get {
        /// Key to retrieve
        key: String,
        /// Topic under which to retrieve key
        #[arg(long)]
        topic: Option<String>,
    },
}
