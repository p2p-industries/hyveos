use hyvectl_commands::families::whoami::Whoami;
use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use crate::boxed_try_stream;
use crate::error::HyveCtlResult;
impl CommandFamily for Whoami {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut discovery = connection.discovery();

        boxed_try_stream! {
            let peer_id = discovery.get_own_id().await?;

            yield CommandOutput::result("whoami")
                .with_field("peer_id", OutputField::PeerId(peer_id))
                .with_tty_template("🤖 You are {peer_id}")
                .with_non_tty_template("{peer_id}")
        }
    }
}