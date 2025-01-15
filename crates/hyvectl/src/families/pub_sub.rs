use futures::{stream::BoxStream, TryStreamExt as _};
use hyvectl_commands::families::pub_sub::PubSub;
use hyveos_core::pub_sub::ReceivedMessage;
use hyveos_sdk::Connection;

use crate::{
    boxed_try_stream,
    error::{HyveCtlError, HyveCtlResult},
    out::CommandOutput,
    util::CommandFamily,
};

impl TryFrom<ReceivedMessage> for CommandOutput {
    type Error = HyveCtlError;

    fn try_from(value: ReceivedMessage) -> Result<Self, Self::Error> {
        let output = CommandOutput::result()
            .with_field("psource", value.propagation_source.to_string())
            .with_field("topic", value.message.topic)
            .with_field("message", String::from_utf8(value.message.data)?)
            .with_field("message_id", String::from_utf8(value.message_id.0)?)
            .with_tty_template("📨 { topic: {topic}, message: {message} }")
            .with_non_tty_template("{topic},{message}");

        match value.source {
            Some(source) => Ok(output
                .with_field("source", source.to_string())
                .with_tty_template("📨 { topic: {topic}, message: {message}, source: {source} }")
                .with_non_tty_template("{topic},{message},{source}")),
            None => Ok(output),
        }
    }
}

impl CommandFamily for PubSub {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut pub_sub_service = connection.pub_sub();

        match self {
            PubSub::Publish { topic, message } => {
                boxed_try_stream! {
                    pub_sub_service.publish(&topic, message.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("message", message)
                        .with_tty_template("📨 Published { {message} } to { {topic} }")
                        .with_non_tty_template("{message},{topic}")
                }
            }
            PubSub::Get { topic } => {
                boxed_try_stream! {
                    yield CommandOutput::spinner("Waiting for Messages...", &["◐", "◒", "◑", "◓"]);

                    let mut message_stream = pub_sub_service.subscribe(&topic).await?;

                    while let Some(event) = message_stream.try_next().await? {
                        yield event.try_into()?
                    }
                }
            }
        }
    }
}
