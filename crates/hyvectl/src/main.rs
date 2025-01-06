mod util;
mod families;
mod output;
mod color;

use hyveos_sdk::{Connection};
use std::io::{stdout, IsTerminal, Write};
use std::time::Duration;
use clap::{Parser};
use util::CommandFamily;
use futures::stream::BoxStream;
use futures::StreamExt;
use indicatif::ProgressStyle;
use crate::output::{CommandOutput, CommandOutputType};
use crate::util::DynError;
use std::path::PathBuf;

use hyvectl_commands::command::{Cli, Families};

impl CommandFamily for Families {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
            Families::PubSub(cmd) => cmd.run(connection).await,
            Families::Inspect(cmd) => cmd.run(connection).await,
            Families::ReqRes(cmd) => cmd.run(connection).await,
            Families::Hyve(cmd) => cmd.run(connection).await,
            Families::File(cmd) => cmd.run(connection).await,
            Families::Whoami(cmd) => cmd.run(connection).await
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let cli = Cli::parse();

    let socket_path = ["/run", "/var/run"]
        .into_iter()
        .map(str::to_string)
        .map(PathBuf::from)
        .chain(Some(std::env::temp_dir()))
        .find_map(|p| p.join("hyved").join("bridge").join("bridge.sock").canonicalize().ok())
        .ok_or(std::io::Error::from(std::io::ErrorKind::NotConnected))?;

    let connection = Connection::builder()
        .custom_socket(socket_path)
        .connect()
        .await?;

    let is_tty = stdout().is_terminal();
    let mut stdout = stdout().lock();

    let theme = if is_tty {Some(color::Theme::default())} else {None};

    let mut output_stream = cli.command.run(&connection).await;

    let mut progress_bar = None;
    let mut spinner = None;

    while let Some(output) = output_stream.next().await {
        let command_output = output?;

        match command_output.output {
            CommandOutputType::Progress(p) => {
                if is_tty {
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
                }
            },
            CommandOutputType::Spinner {message, tick_strings} => {
                if cli.json {
                    continue
                }
                let sp = indicatif::ProgressBar::new_spinner();

                let tick_slices: Vec<&str> = tick_strings.iter().map(|s| s.as_str()).collect();

                sp.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&tick_slices)
                        .template("[{spinner:.green}] {msg}")
                        .expect("Failed to set spinner template")
                );

                sp.enable_steady_tick(Duration::from_millis(100));
                sp.set_message(message);

                spinner = Some(sp);
            },
            _ => {
                if cli.json {
                    command_output.write_json(&mut stdout)?;
                } else {
                    if let Some(sp) = &spinner {
                        command_output.write_to_spinner(sp, &theme)?;
                    } else {
                        command_output.write(&mut stdout, &theme)?;
                    }
                }
            }
        }

        if !is_tty {
            match stdout.flush() {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::BrokenPipe {
                        return Ok(())
                    } else {
                        Err(e)?
                    }
                }
            }
        }
    }

    if let Some(pb) = progress_bar.take() {
        pb.finish_and_clear();
    }

    if let Some(sp) = spinner.take() {
        sp.finish_and_clear();
    }

    Ok(())
}
