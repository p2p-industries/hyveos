use clap::Subcommand;

#[derive(Subcommand)]
pub enum Hyve {
    /// Starts an application on a given node
    Start {
        /// Image to deploy
        image: String,

        /// Peer-Id of the target node
        #[arg(conflicts_with = "local")]
        peer: Option<String>,

        /// Ports to expose from target to application
        #[arg(long)]
        ports: Vec<u16>,

        /// Deploy the image locally
        #[arg(long, conflicts_with = "peer")]
        local: bool,

        /// Deploy the image persistent, meaning it will be restarted when the runtime is restarted
        #[arg(long)]
        persistent: bool,
    },

    /// List running applications on a given node
    List {
        /// Peer-Id of the target node
        #[arg(conflicts_with = "local")]
        peer: Option<String>,

        /// List local running scripts
        #[arg(long, conflicts_with = "peer")]
        local: bool,
    },

    /// Stops an application on a given node
    Stop {
        /// Identifier of the script to stop
        id: String,

        /// Peer-Id of the target node
        peer: Option<String>,

        /// List local running scripts
        #[arg(long, conflicts_with = "peer")]
        local: bool,
    },
}
