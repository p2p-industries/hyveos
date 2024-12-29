use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{resolve_stream, CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream};
use futures::stream::BoxStream;
use hyvectl_commands::families::kv::Kv;


impl CommandFamily for Kv {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut dht = connection.dht();

        match self {
            Kv::Put { key, value, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    dht.put_record(topic.clone(), key.clone(), value.clone()).await?;
                    Ok(CommandOutput::new_result("KV Put")
                        .with_field("key", OutputField::String(key))
                        .with_field("value", OutputField::String(value))
                        .with_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Added {value} to {key} under topic {topic}")
                    )
                }).boxed()
            },
            Kv::Get { key, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    let result = dht.get_record(topic.clone(), key.clone()).await?;
                    Ok(match result {
                        Some(res) => CommandOutput::new_result("KV Get")
                            .with_field("key", OutputField::String(key))
                            .with_field("topic", OutputField::String(topic))
                            .with_field("value", OutputField::String(String::from_utf8(res)?))
                            .with_human_readable_template("Retrieved {value} for {key} in topic {topic}"),
                        None => CommandOutput::new_result("KV Get")
                            .with_field("key", OutputField::String(key))
                            .with_field("topic", OutputField::String(topic))
                            .with_human_readable_template("Unable to retrieve key {key} in topic {topic}")
                    })
                }).boxed()
            },
            Kv::Provide { key, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    dht.provide(topic.clone(), key.clone()).await?;
                    Ok(CommandOutput::new_result("KV Provide")
                        .with_field("key", OutputField::String(key))
                        .with_field("topic", OutputField::String(topic))
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
                        CommandOutput::new_result("KV GetProviders")
                            .with_field("key", OutputField::String(key.clone()))
                            .with_field("provider", OutputField::String(provider_id.to_string()))
                            .with_field("topic", OutputField::String(topic.clone()))
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
