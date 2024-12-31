use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::kv::Kv;
use crate::boxed_try_stream;

impl CommandFamily for Kv {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut dht = connection.dht();

        match self {
            Kv::Put { key, value, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    dht.put_record(topic.clone(), key.clone(), value.clone()).await?;

                    yield CommandOutput::result("KV Put")
                        .with_field("key", OutputField::String(key.clone()))
                        .with_field("value", OutputField::String(value.clone()))
                        .with_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Added {value} to {key} under topic {topic}");
                }
            },
            Kv::Get { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();
                    let result = dht.get_record(topic.clone(), key.clone()).await?;

                    match result {
                        Some(res) => yield CommandOutput::result("KV Get")
                            .with_field("key", OutputField::String(key))
                            .with_field("topic", OutputField::String(topic))
                            .with_field("value", OutputField::String(String::from_utf8(res)?))
                            .with_human_readable_template("Retrieved {value} for {key} in topic {topic}"),
                        None => yield CommandOutput::result("KV Get")
                            .with_field("key", OutputField::String(key))
                            .with_field("topic", OutputField::String(topic))
                            .with_human_readable_template("Unable to retrieve key {key} in topic {topic}")
                    }
                }
            },
            Kv::Provide { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    dht.provide(topic.clone(), key.clone()).await?;

                    yield CommandOutput::result("KV Provide")
                        .with_field("key", OutputField::String(key))
                        .with_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Started providing key {key} in topic {topic}")
                }
            },
            Kv::GetProviders { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    let mut providers_stream = dht.get_providers(topic.clone(), key.clone()).await?;

                    while let Some(event) = providers_stream.next().await {

                        match event {
                            Ok(provider) => yield CommandOutput::result("KV GetProviders")
                                .with_field("key", OutputField::String(key.clone()))
                                .with_field("provider", OutputField::String(provider.to_string()))
                                .with_field("topic", OutputField::String(topic.clone()))
                                .with_human_readable_template("Provider {provider} found for {key} in topic {topic}"),
                            Err(e) => yield CommandOutput::error("KV GetProviders", &e.to_string())
                        }
                    }
                }
            },
            Kv::StopProvide { key: _, topic: _ } => {
                todo!()
            }
        }
    }
}
