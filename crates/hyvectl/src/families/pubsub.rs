use hyvectl_commands::families::pubsub::PubSub;
use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyveos_core::gossipsub::ReceivedMessage;
use crate::boxed_try_stream;
use crate::error::{HyveCtlError, HyveCtlResult};

impl TryFrom<ReceivedMessage> for CommandOutput {
    type Error = HyveCtlError;

    fn try_from(value: ReceivedMessage) -> Result<Self, Self::Error> {
        let output = CommandOutput::result("")
            .with_field("psource", value.propagation_source.into())
            .with_field("topic", value.message.topic.into())
            .with_field("message", String::from_utf8(value.message.data)?.into())
            .with_field("message_id", String::from_utf8(value.message_id.0)?.into())
            .with_tty_template("📨 {{ topic: {topic}, message: {message} }}")
            .with_non_tty_template("{topic},{message}");

        match value.source {
            Some(source) => {
                Ok(output.with_field("source", source.into())
                    .with_non_tty_template("{topic},{message},{source}"))
            },
            None => {
                Ok(output)
            }
        }
    }
}


impl CommandFamily for PubSub {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut pubsub = connection.gossipsub();

        match self {
            PubSub::Publish { topic, message } => {
                boxed_try_stream! {
                    pubsub.publish(topic.clone(), message.clone()).await?;

                    yield CommandOutput::result("pub-sub/publish")
                        .with_field("topic", OutputField::String(topic.clone()))
                        .with_field("message", OutputField::String(message.clone()))
                        .with_tty_template("📨 Published {message} to topic {topic}")
                        .with_non_tty_template("{message},{topic}")
                }
            },
            PubSub::Get { topic } => {
                boxed_try_stream! {
                    let mut message_stream = pubsub.subscribe(topic.clone()).await?;

                    yield CommandOutput::spinner("Waiting for Messages...", &["◐", "◒", "◑", "◓"]);

                    while let Some(event) = message_stream.next().await {
                        yield event?.try_into()?
                    }
                }
            }
        }
    }
}
