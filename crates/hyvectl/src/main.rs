mod util;
mod families;
mod output;

use families::{kv};

use clap::{Parser, Subcommand};
use p2p_industries_sdk::P2PConnection;
use std::error::Error;
use std::io::{stdout, IsTerminal, Write};
use util::CommandFamily;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use crate::output::CommandOutput;

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl", version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Families,
}

#[derive(Subcommand)]
enum Families {
    #[command(subcommand, about = "Key-Value Store")]
    KV(kv::Kv),
}


#[async_trait]
impl CommandFamily for Families {
    async fn run(self, connection: &P2PConnection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let connection = P2PConnection::get().await?;

    let is_tty = stdout().is_terminal();

    let mut output_stream = cli.command.run(&connection).await;

    while let Some(output) = output_stream.next().await {
        let command_output = output?;
        let mut out = stdout();
        command_output.write(&mut out)?;

        if !is_tty {
            out.flush()?;
        }
    }

    Ok(())
}
