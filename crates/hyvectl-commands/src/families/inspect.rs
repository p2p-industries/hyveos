use clap::Subcommand;

#[derive(Subcommand)]
pub enum Inspect {
    #[command(about = "Retrieves a stream of mesh topology events")]
    Mesh {
        #[arg(help = "Only retrieves local mesh topology events", long)]
        local: Option<bool>,
    },
    #[command(about = "Retrieves a stream of service debug events")]
    Services
}