use std::path::Path;
use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::file::File;
use hyveos_core::file_transfer::{Cid, DownloadEvent};
use crate::util::{CommandFamily, DynError};
use hyveos_sdk::Connection;
use crate::output::{CommandOutput, OutputField};
use crate::single_output_stream;

impl CommandFamily for File {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut file_transfer_service = connection.file_transfer();

        match self {
            File::Publish {path} => {
                let cid = file_transfer_service.publish_file(Path::new(&path))
                    .await.unwrap();

                single_output_stream!(
                    CommandOutput::new_result("File Publish")
                    .with_field("cid", OutputField::String(cid.to_string()))
                    .with_human_readable_template("Published file under cid {cid}")
                )
            }
            File::Get {cid, out } => {
                let output_stream = async_stream::try_stream! {
                    let mut download_stream = file_transfer_service
                        .get_file_with_progress(cid.parse::<Cid>().unwrap())
                        .await.unwrap();

                    yield CommandOutput::new_message("File Get", "Starting Download...");

                    while let Some(event) = download_stream.next().await {
                        let event = event?;

                        match event {
                            DownloadEvent::Progress(p) => {
                                yield CommandOutput::new_progress("File Get", p)
                            }
                            DownloadEvent::Ready(path) => {
                                yield CommandOutput::new_result("File Get")
                                .with_field("cid", OutputField::String(cid.to_string()))
                                .with_field("local_path", OutputField::String(path.display().to_string()))
                                .with_human_readable_template("Downloaded file with cid {cid} to {local_path}");
                            }
                        }
                    }
                };

                output_stream.boxed()
            }
        }
    }
}