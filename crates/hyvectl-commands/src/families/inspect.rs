use clap::Subcommand;

#[derive(Subcommand)]
pub enum Inspect {
    /// Retrieves a stream of mesh topology events
    Mesh {
        /// Only retrieves local mesh topology events
        #[arg(long)]
        local: Option<bool>,
    },
    /// Retrieves a stream of service debug events
    Services,
}
