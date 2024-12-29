use hyvectl_commands::families::pubsub::PubSub;
use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{resolve_stream, CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream};
use futures::stream::BoxStream;

impl CommandFamily for PubSub {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut pubsub = connection.gossipsub();

        match self {
            PubSub::Publish { topic, message } => {
                stream::once(async move {
                    pubsub.publish(topic.clone(), message.clone()).await?;
                    Ok(CommandOutput::new_result("PubSub Publish")
                        .with_field("topic", OutputField::String(topic))
                        .with_field("message", OutputField::String(message))
                        .with_human_readable_template("Published {message} to topic {topic}"))
                }).boxed()
            },
            PubSub::Get { topic, n, follow } => {
                let subscription_stream = resolve_stream(
                    pubsub.subscribe(topic.clone()).await).await;

                let subscription_stream = match n {
                    Some(n) => { subscription_stream.take(n as usize).boxed() }
                    None => { subscription_stream.boxed() }
                };

                subscription_stream
                    .map_ok(move |message|
                        {CommandOutput::new_result("PubSub Subscribe")
                            .with_field("message", OutputField::GossipMessage(message.clone()))
                            .with_human_readable_template("Received message {message}")})
                    .map_err(|e| e.into())
                    .boxed()
            }
        }
    }
}
