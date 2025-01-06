use clap::Subcommand;

#[derive(Subcommand)]
pub enum ReqRes {
    /// Send the message to a given peer and returns its response
    Send {
        /// Target peer
        peer: String,
        /// Request
        request: String,
        /// Topic under which to send the request
        #[arg(long)]
        topic: Option<String>,
    },

    /// Retrieve a stream of messages from peers
    Receive {},

    /// Respond to messages from peers
    Respond {
        /// ID of the associated request
        id: u64,
        /// Response
        response: String,
    }
}

