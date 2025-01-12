use futures::{stream::BoxStream, StreamExt};
use hyvectl_commands::families::whoami::Whoami;
use hyveos_sdk::Connection;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};
impl CommandFamily for Whoami {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut discovery = connection.discovery();

        boxed_try_stream! {
            let peer_id = discovery.get_own_id().await?;

            yield CommandOutput::result()
                .with_field("peer_id", peer_id.to_string())
                .with_tty_template("🤖 You are { {peer_id} }")
                .with_non_tty_template("{peer_id}")
        }
    }
}
