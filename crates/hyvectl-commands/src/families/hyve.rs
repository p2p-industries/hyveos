use clap::Subcommand;

#[derive(Subcommand)]
pub enum Hyve {
    #[command(about = "Starts an application on a given node")]
    Start {
        image: String,
        #[arg(help = "Peer-Id of the target node", long)]
        peer: String,
        #[arg(help = "Deploys the image locally", long)]
        local: Option<bool>,
        #[arg(help = "Ports to expose from target to application", long)]
        ports: Vec<u16>,

    },
    #[command(about = "List running applications on a given node")]
    List {
        peer: String
    },
    #[command(about = "Stops an application on a give node")]
    Stop {
        peer: String,
        container: String,
    }
}