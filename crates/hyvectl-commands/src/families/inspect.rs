use clap::Subcommand;

#[derive(Subcommand)]
pub enum Inspect {
    #[command(about = "Retrieves a stream of mesh topology events")]
    Mesh {
        /// Only retrieves local mesh topology events
        #[arg(long)]
        local: Option<bool>,
    },
    #[command(about = "Retrieves a stream of service debug events")]
    Services
}