mod util;
mod families;
mod output;

use hyveos_sdk::{Connection};
use std::error::Error;
use std::io::{stdout, IsTerminal, Write};
use clap::Parser;
use util::CommandFamily;
use futures::stream::BoxStream;
use futures::StreamExt;
use crate::output::CommandOutput;

use hyvectl_commands::command::{Cli, Families};

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