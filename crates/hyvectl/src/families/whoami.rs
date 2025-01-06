use hyvectl_commands::families::whoami::Whoami;
use hyveos_sdk::Connection;
use crate::util::{CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use crate::boxed_try_stream;

impl CommandFamily for Whoami {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut discovery = connection.discovery();

        boxed_try_stream! {
            let peer_id = discovery.get_own_id().await?;

            yield CommandOutput::result("whoami")
                .with_field("peer_id", OutputField::PeerId(peer_id))
                .with_human_readable_template("🤖 You are {peer_id}")
        }
    }
}