use clap::Subcommand;

#[derive(Subcommand)]
pub enum Discovery {
    /// Announce that this node is providing a discovery key
    Provide {
        /// Key to provide
        key: String,
        /// Topic under which to provide key
        #[arg(long)]
        topic: Option<String>,
    },
    /// Get the providers for the given discovery key
    GetProviders {
        /// Key to get providers for
        key: String,
        /// Topic under which to get providers for key
        #[arg(long)]
        topic: Option<String>,
    },
    /// Stop providing a given discovery key
    StopProvide {
        /// Key to stop providing
        key: String,
        /// Topic under which to stop providing key
        #[arg(long)]
        topic: Option<String>,
    },
}
