use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum File {
    /// Publishes a file into the file network
    Publish {
        /// Path to file
        path: PathBuf,
    },
    /// Retrieves a file from the file network
    Get {
        /// Cid of file in network
        cid: String,
        /// Output path
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}
