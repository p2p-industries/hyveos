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
    /// Announce that this node can provide the value for the given key
    Provide {
        /// Key to provide
        key: String,
        /// Topic under which to provide key
        #[arg(long)]
        topic: Option<String>,
    },
    /// Get the providers for the given key
    GetProviders {
        /// Key to get providers for
        key: String,
        /// Topic under which to get providers for key
        #[arg(long)]
        topic: Option<String>,
    },
    /// Stop providing a given key
    StopProvide {
        /// Key to stop providing
        key: String,
        /// Topic under which to stop providing key
        #[arg(long)]
        topic: Option<String>,
    },
}
