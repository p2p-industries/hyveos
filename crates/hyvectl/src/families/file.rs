use futures::{stream::BoxStream, TryStreamExt as _};
use hyvectl_commands::families::file::File;
use hyveos_core::file_transfer::{Cid, DownloadEvent};
use hyveos_sdk::Connection;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for File {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut file_transfer_service = connection.file_transfer();

        match self {
            File::Publish { path } => {
                boxed_try_stream! {
                    let input_path = path.canonicalize()?;

                    yield CommandOutput::spinner("Publishing File...", &["◐", "◒", "◑", "◓"]);

                    let cid = file_transfer_service.publish(input_path)
                    .await?;

                    yield CommandOutput::result()
                    .with_field("cid", cid.to_string())
                    .with_tty_template("Published file as { {cid} }")
                    .with_non_tty_template("{cid}")
                }
            }
            File::Get { cid, out } => {
                boxed_try_stream! {
                    let mut download_stream = file_transfer_service
                        .get_with_progress(cid.parse::<Cid>()?)
                        .await?;

                    yield CommandOutput::message("Starting Download...");

                    while let Some(event) = download_stream.try_next().await? {
                        match event {
                            DownloadEvent::Progress(p) => {
                                yield CommandOutput::progress(p)
                            }
                            DownloadEvent::Ready(path) => {
                                let path = match out.clone() {
                                    Some(o) => {
                                        tokio::fs::copy(path, &o).await?;
                                        o
                                    },
                                    None => path
                                };

                                yield CommandOutput::result()
                                .with_field("cid", cid.to_string())
                                .with_field("local_path", path.display().to_string())
                                .with_tty_template("Downloaded file to { {local_path} }")
                                .with_non_tty_template("{cid},{local_path}");
                            }
                        }
                    }
                }
            }
        }
    }
}
