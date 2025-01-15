use clap::Subcommand;

#[derive(Subcommand)]
pub enum Debug {
    /// Retrieves a stream of mesh topology events
    Mesh,
    /// Retrieves a stream of service debug events
    Services,
}
