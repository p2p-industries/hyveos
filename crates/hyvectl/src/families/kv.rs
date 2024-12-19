use hyveos_sdk::Connection;
use std::error::Error;
use colored::Colorize;
use crate::util::{resolve_stream, CommandFamily};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream};
use futures::stream::BoxStream;
use hyvectl_commands::families::kv::Kv;


impl CommandFamily for Kv {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        let mut dht = connection.dht();

        match self {
            Kv::Put { key, value, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    dht.put_record(topic.clone(), key.clone(), value.clone()).await?;
                    Ok(CommandOutput::new("KV Put")
                        .add_field("key", OutputField::String(key))
                        .add_field("value", OutputField::String(value))
                        .add_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Added {value} to {key} under topic {topic}")
                    )
                }).boxed()
            },
            Kv::Get { key, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    let result = dht.get_record(topic.clone(), key.clone()).await?;
                    Ok(match result {
                        Some(res) => CommandOutput::new("KV Get")
                            .add_field("key", OutputField::String(key))
                            .add_field("topic", OutputField::String(topic))
                            .add_field("value", OutputField::String(String::from_utf8(res)?))
                            .with_human_readable_template("Retrieved {value} for {key} in topic {topic}"),
                        None => CommandOutput::new("KV Get")
                            .add_field("key", OutputField::String(key))
                            .add_field("topic", OutputField::String(topic))
                            .with_human_readable_template("Unable to retrieve key {key} in topic {topic}")
                    })
                }).boxed()
            },
            Kv::Provide { key, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    dht.provide(topic.clone(), key.clone()).await?;
                    Ok(CommandOutput::new("KV Provide")
                        .add_field("key", OutputField::String(key))
                        .add_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Started providing key {key} in topic {topic}")
                    )
                }).boxed()
            },
            Kv::GetProviders { key, topic } => {
                let topic = topic.unwrap_or_default();
                let providers_stream = resolve_stream(
                    dht.get_providers(topic.clone(), key.clone()).await).await;

                providers_stream
                    .map_ok(move |provider_id| {
                        CommandOutput::new("KV GetProviders")
                            .add_field("key", OutputField::String(key.clone()))
                            .add_field("provider", OutputField::String(provider_id.to_string()))
                            .add_field("topic", OutputField::String(topic.clone()))
                            .with_human_readable_template("Provider {provider} found for {key} in topic {topic}")
                    })
                    .map_err(|e| e.into())
                    .boxed()
            },
            Kv::StopProvide { key: _, topic: _ } => {
                todo!()
            }
        }
    }
}