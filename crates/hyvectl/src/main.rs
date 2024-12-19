mod util;
mod families;
mod output;
mod color;

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
            Families::PubSub(cmd) => cmd.run(connection).await,
            Families::Inspect(cmd) => cmd.run(connection).await,
            Families::ReqRes(cmd) => cmd.run(connection).await,
            Families::Whoami => {todo!()}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let socket_path = hyveos_core::get_runtime_base_path()
        .join("bridge")
        .join("bridge.sock");

    let connection = Connection::builder()
        .custom_socket(socket_path)
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
