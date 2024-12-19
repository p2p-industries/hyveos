use clap::Subcommand;

#[derive(Subcommand)]
pub enum ReqRes {
    #[command(about = "Send the message to a given peer")]
    Send {
        peer: String,
        message: String,
        #[arg(long)]
        topic: Option<String>,
    },

    #[command(about = "Retrieve a stream of messages from peers")]
    Receive {},

    #[command(about = "Respond to messages from peers")]
    Respond {
        id: u64,
        message: String,
    }
}

