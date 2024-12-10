mod util;
mod families;
mod output;

use families::{kv};

use clap::{Parser, Subcommand};
use hyveos_sdk::{Connection};
use std::error::Error;
use std::io::{stdout, IsTerminal, Write};
use util::CommandFamily;
use futures::stream::BoxStream;
use futures::StreamExt;
use crate::output::CommandOutput;

#[derive(Parser)]
#[command(name = "hyvectl", about = "Hyvectl")]
struct Cli {
    #[command(subcommand)]
    command: Families,
}

#[derive(Subcommand)]
enum Families {
    #[command(subcommand, about = "Key-Value Store")]
    KV(kv::Kv),
}


impl CommandFamily for Families {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let connection = Connection::builder()
        .connect()
        .await?;

    let is_tty = stdout().is_terminal();
    let mut stdout = stdout().lock();

    let mut output_stream = cli.command.run(&connection).await;

    while let Some(output) = output_stream.next().await {
        let command_output = output?;

        command_output.write(&mut stdout)?;

        if !is_tty {
            stdout.flush()?;
        }
    }

    Ok(())
}
