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
use crate::output::{CommandOutput, CommandOutputType};
use crate::util::DynError;

use hyvectl_commands::command::{Cli, Families};

impl CommandFamily for Families {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
            Families::PubSub(cmd) => cmd.run(connection).await,
            Families::Inspect(cmd) => cmd.run(connection).await,
            Families::ReqRes(cmd) => cmd.run(connection).await,
            Families::File(cmd) => cmd.run(connection).await,
            Families::Whoami(cmd) => cmd.run(connection).await
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
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

    let mut progress_bar = None;

    while let Some(output) = output_stream.next().await {
        let command_output = output?;

        match command_output.output {
            CommandOutputType::Progress(p) => {
                if progress_bar.is_none() {
                    let pb = indicatif::ProgressBar::new(100);
                    pb.set_style(indicatif::ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}")
                        .unwrap());
                    progress_bar = Some(pb);
                }
                if let Some(pb) = &progress_bar {
                    pb.set_position(p)
                }
            },
            _ => {
                command_output.write(&mut stdout)?;
            }
        }

        if !is_tty {
            stdout.flush()?;
        }
    }

    if let Some(pb) = progress_bar.take() {
        pb.finish_and_clear();
    }

    Ok(())
}
