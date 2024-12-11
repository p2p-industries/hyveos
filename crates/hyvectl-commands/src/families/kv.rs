use clap::Subcommand;

#[derive(Subcommand)]
pub enum Kv {
    #[command(about = "Publish the value for the given key")]
    Put {
        key: String,
        value: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the value for the given key")]
    Get {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Announce that this node can provide the value for the given key")]
    Provide {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the providers for the given key")]
    GetProviders {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Stop providing a given key")]
    StopProvide {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    }
}