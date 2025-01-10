use clap::Subcommand;

#[derive(Subcommand)]
pub enum File {
    /// Publishes a file into the file network
    Publish {
        /// Path to file
        path: String,
    },
    /// Retrieves a file from the file network
    Get {
        /// Cid of file in network
        cid: String,
        /// Output path
        #[arg(long)]
        out: Option<String>,
    },
}
