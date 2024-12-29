use clap::Subcommand;

#[derive(Subcommand)]
pub enum Hyve {
    #[command(about = "Starts an application on a given node")]
    Start {
        image: String,

        /// Peer-Id of the target node
        #[arg(long, conflicts_with = "local")]
        peer: Option<String>,

        /// Deploy the image locally
        #[arg(long, conflicts_with = "peer")]
        local: bool,

        /// Ports to expose from target to application
        #[arg(long)]
        ports: Vec<u16>,
    },

    #[command(about = "List running applications on a given node")]
    List {
        /// Peer-Id of the target node
        #[arg(long, conflicts_with = "local")]
        peer: Option<String>,

        /// List local running scripts
        #[arg(long, conflicts_with = "peer")]
        local: bool,
    },

    #[command(about = "Stops an application on a given node")]
    Stop {
        /// Peer-Id of the target node
        #[arg(long, conflicts_with = "local")]
        peer: Option<String>,

        /// List local running scripts
        #[arg(long, conflicts_with = "peer")]
        local: bool,

        /// Identifier of the script to stop
        #[arg(long)]
        id: String,
    }
}
