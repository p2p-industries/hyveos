mod color;
mod error;
mod families;
mod out;
mod util;

use crate::error::{HyveCtlError, HyveCtlResult};
use crate::out::CommandOutput;
use clap::Parser;
use futures::stream::BoxStream;
use futures::StreamExt;
use hyvectl_commands::command::{Cli, Families};
use hyveos_sdk::Connection;
use indicatif::ProgressStyle;
use miette::{Context, IntoDiagnostic};
use std::io::{stdout, IsTerminal, Write};
use std::path::PathBuf;
use std::time::Duration;
use util::CommandFamily;

impl CommandFamily for Families {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        match self {
            Families::KV(cmd) => cmd.run(connection).await,
            Families::PubSub(cmd) => cmd.run(connection).await,
            Families::Inspect(cmd) => cmd.run(connection).await,
            Families::ReqRes(cmd) => cmd.run(connection).await,
            Families::Hyve(cmd) => cmd.run(connection).await,
            Families::File(cmd) => cmd.run(connection).await,
            Families::Whoami(cmd) => cmd.run(connection).await,
        }
    }
}

fn find_hyved_endpoint(endpoint: &str) -> miette::Result<PathBuf> {
    let candidates = ["/run", "/var/run"]
        .into_iter()
        .map(PathBuf::from)
        .chain(std::iter::once(std::env::temp_dir()));

    for dir in candidates {
        let candidate = dir.join("hyved").join("bridge").join(endpoint);

        if let Ok(path) = candidate
            .canonicalize()
            .into_diagnostic()
            .wrap_err("Unable to connect to hyveOS bridge")
        {
            return Ok(path);
        }
    }

    Err(miette::miette!("No possible path to hyveOS Bridge sock"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let socket_path = find_hyved_endpoint("bridge.sock")?;
    let shared_dir_path = find_hyved_endpoint("files")?;

    let connection = Connection::builder()
        .custom(socket_path, shared_dir_path)
        .connect()
        .await
        .map_err(HyveCtlError::from)?;

    let is_tty = stdout().is_terminal();
    let mut stdout = stdout().lock();

    let theme = if is_tty {
        Some(color::Theme::default())
    } else {
        None
    };

    let mut output_stream = cli.command.run(&connection).await;

    let mut progress_bar = None;
    let mut spinner = None;

    while let Some(output) = output_stream.next().await {
        let command_output = output?;

        match command_output {
            CommandOutput::Progress(p) => {
                if is_tty {
                    if progress_bar.is_none() {
                        let pb = indicatif::ProgressBar::new(100);
                        pb.set_style(
                            indicatif::ProgressStyle::default_bar()
                                .template(
                                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}",
                                )
                                .unwrap(),
                        );
                        progress_bar = Some(pb);
                    }
                    if let Some(pb) = &progress_bar {
                        pb.set_position(p);
                    }
                }
            }
            CommandOutput::Spinner {
                message,
                tick_strings,
            } => {
                if !is_tty || cli.json {
                    continue;
                }

                let sp = indicatif::ProgressBar::new_spinner();

                let tick_slices: Vec<&str> = tick_strings
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();

                sp.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&tick_slices)
                        .template("[{spinner:.green}] {msg}")
                        .expect("Failed to set spinner template"),
                );

                sp.enable_steady_tick(Duration::from_millis(100));
                sp.set_message(message);

                spinner = Some(sp);
            }
            _ => {
                if cli.json {
                    command_output
                        .write_json(&mut stdout, is_tty)
                        .map_err(HyveCtlError::from)?;
                } else if let Some(sp) = &spinner {
                    command_output
                        .write_to_spinner(sp, theme.as_ref(), is_tty)
                        .map_err(HyveCtlError::from)?;
                } else {
                    command_output
                        .write(&mut stdout, theme.as_ref(), is_tty)
                        .map_err(HyveCtlError::from)?;
                }
            }
        }

        if !is_tty {
            match stdout.flush() {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => return Ok(()),
                Err(e) => Err(HyveCtlError::from(e))?,
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
