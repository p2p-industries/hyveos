use hyvectl_commands::families::pubsub::PubSub;
use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{resolve_stream, CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream, FutureExt};
use futures::stream::BoxStream;

impl CommandFamily for PubSub {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut pubsub = connection.gossipsub();

        match self {
            PubSub::Publish { topic, message } => {
                let stream = async_stream::try_stream! {
                    pubsub.publish(topic.clone(), message.clone()).await?;

                    yield CommandOutput::result("PubSub Publish")
                        .with_field("topic", OutputField::String(topic.clone()))
                        .with_field("message", OutputField::String(message.clone()))
                        .with_human_readable_template("Published {message} to topic {topic}")
                };

                stream.boxed()
            },
            PubSub::Get { topic, n, follow } => {
                let stream = async_stream::try_stream! {
                    let mut message_stream = pubsub.subscribe(topic.clone()).await?;
                    let mut count = 0;

                    yield CommandOutput::message("PubSub Subscribe", "Listening on Topic");

                    while let Some(event) = message_stream.next().await {
                        if let Some(limit) = n {
                            if count >= limit {
                                break;
                            }
                        }

                        match event {
                            Ok(message) => {
                                yield CommandOutput::result("PubSub Subscribe")
                                        .with_field("message", OutputField::GossipMessage(message.clone()))
                                        .with_human_readable_template("Received message {message}");
                                count += 1;
                            },
                            Err(e) =>
                                yield CommandOutput::error("PubSub Subscribe", &e.to_string())
                        }
                    }
                };

                stream.boxed()
            }
        }
    }
}
