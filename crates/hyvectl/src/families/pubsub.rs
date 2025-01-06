use hyvectl_commands::families::pubsub::PubSub;
use hyveos_sdk::Connection;
use crate::util::{CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use crate::boxed_try_stream;

impl CommandFamily for PubSub {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut pubsub = connection.gossipsub();

        match self {
            PubSub::Publish { topic, message } => {
                boxed_try_stream! {
                    pubsub.publish(topic.clone(), message.clone()).await?;

                    yield CommandOutput::result("pub-sub/publish")
                        .with_field("topic", OutputField::String(topic.clone()))
                        .with_field("message", OutputField::String(message.clone()))
                        .with_human_readable_template("Published {message} to topic {topic}")
                }
            },
            PubSub::Get { topic } => {
                boxed_try_stream! {
                     let mut message_stream = pubsub.subscribe(topic.clone()).await?;
                    let mut count = 0;

                    yield CommandOutput::spinner("Waiting for Messages...", &["◐", "◒", "◑", "◓"]);

                    while let Some(event) = message_stream.next().await {
                        match event {
                            Ok(message) => {
                                yield CommandOutput::result("pub-sub/get")
                                        .with_field("message", OutputField::ReceivedGossipMessage(message.clone()))
                                        .with_human_readable_template("📨 {message}");
                                count += 1;
                            },
                            Err(e) =>
                                yield CommandOutput::error("pub-sub/get", &e.to_string())
                        }
                    }
                }
            }
        }
    }
}
