use hyvectl_commands::families::pubsub::PubSub;
use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::CommandFamily;
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream};
use futures::stream::BoxStream;

impl CommandFamily for PubSub {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        let mut pubsub = connection.gossipsub();

        match self {
            PubSub::Publish { topic, message } => {
                stream::once(async move {
                    pubsub.publish(topic.clone(), message.clone()).await?;
                    Ok(CommandOutput::new("PubSub Publish")
                        .add_field("topic", OutputField::String(topic))
                        .add_field("message", OutputField::String(message))
                        .with_human_readable_template("Published {message} to topic {topic}"))
                }).boxed()
            },
            PubSub::Get { topic, n, follow } => {
                let mut subscription_future = pubsub.subscribe(topic.clone());
                
                let subscription_stream = match subscription_future.await {
                    Ok(s) => s,
                    Err(e) => {
                        return stream::once(async move { Err(e.into()) }).boxed()
                    }
                };

                let subscription_stream = match n {
                    Some(n) => { subscription_stream.take(n as usize).boxed() }
                    None => { subscription_stream.boxed() }
                };

                subscription_stream
                    .map_ok(move |message|
                        {CommandOutput::new("PubSub Subscribe")
                            .add_field("message", OutputField::GossipMessage(message.clone()))
                            .with_human_readable_template("Received message {message}")})
                    .map_err(|e| e.into())
                    .boxed()
            }
        }
    }
}
