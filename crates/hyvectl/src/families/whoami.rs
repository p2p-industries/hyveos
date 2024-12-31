use hyvectl_commands::families::whoami::Whoami;
use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{resolve_stream, CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream, FutureExt};
use futures::stream::BoxStream;
use crate::boxed_try_stream;

impl CommandFamily for Whoami {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut discovery = connection.discovery();

        boxed_try_stream! {
            let peer_id = discovery.get_own_id().await?;

            yield CommandOutput::result("WhoAmI")
                .with_field("peer_id", OutputField::String(peer_id.to_string()))
                .with_human_readable_template("You are {peer_id}")
        }
    }
}