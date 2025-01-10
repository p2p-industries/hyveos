use crate::boxed_try_stream;
use crate::error::HyveCtlResult;
use crate::out::CommandOutput;
use crate::util::CommandFamily;
use futures::stream::BoxStream;
use futures::StreamExt;
use hyvectl_commands::families::whoami::Whoami;
use hyveos_sdk::Connection;
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
