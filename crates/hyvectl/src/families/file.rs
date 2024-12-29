use std::error::Error;
use std::path::Path;
use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::file::File;
use crate::util::CommandFamily;
use hyveos_sdk::Connection;
use crate::output::{CommandOutput, OutputField};
use crate::single_output_stream;

impl CommandFamily for File {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        let mut file_transfer_service = connection.file_transfer();

        match self {
            File::Publish {path} => {
                let cid = file_transfer_service.publish_file(
                    Path::new(&path)).await.unwrap();

                single_output_stream!(CommandOutput::new("File Publish")
                .add_field("path", OutputField::String(path))
                .add_field("cid", OutputField::Cid(cid))
                .with_human_readable_template("Published {path} with cid {cid}"))
            }
            File::Get {cid, out } => {
                todo!()
            }
        }
    }
}