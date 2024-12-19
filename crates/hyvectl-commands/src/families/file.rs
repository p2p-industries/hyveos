use clap::Subcommand;

#[derive(Subcommand)]
pub enum File {
    #[command(about = "Publishes a file into the file network")]
    Publish {
        path: String
    },
    #[command(about = "Retrieves a file from the file network")]
    Get {
        cid: String,
        #[arg(long)]
        out: Option<String>,
    }
}