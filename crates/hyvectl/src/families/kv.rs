use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::kv::Kv;
use crate::boxed_try_stream;
use crate::error::HyveCtlResult;
impl CommandFamily for Kv {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut dht = connection.dht();

        match self {
            Kv::Put { key, value, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    dht.put_record(topic.clone(), key.clone(), value.clone()).await?;

                    yield CommandOutput::result("kv/put")
                        .with_field("topic", OutputField::String(topic))
                        .with_field("key", OutputField::String(key.clone()))
                        .with_field("value", OutputField::String(value.clone()))
                        .with_human_readable_template("Added {value} to {key} under topic {topic}");
                }
            },
            Kv::Get { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();
                    let result = dht.get_record(topic.clone(), key.clone()).await?;

                    match result {
                        Some(res) => yield CommandOutput::result("kv/get")
                            .with_field("topic", OutputField::String(topic))
                            .with_field("key", OutputField::String(key))
                            .with_field("value", OutputField::String(String::from_utf8(res)?))
                            .with_human_readable_template("Retrieved {value} for {key} in topic {topic}"),
                        None => yield CommandOutput::result("kv/get")
                            .with_field("topic", OutputField::String(topic))
                            .with_field("key", OutputField::String(key))
                            .with_human_readable_template("Unable to retrieve key {key} in topic {topic}")
                    }
                }
            },
            Kv::Provide { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    dht.provide(topic.clone(), key.clone()).await?;

                    yield CommandOutput::result("kv/provide")
                        .with_field("topic", OutputField::String(topic))
                        .with_field("key", OutputField::String(key))
                        .with_human_readable_template("Started providing key {key} in topic {topic}")
                }
            },
            Kv::GetProviders { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    let mut providers_stream = dht.get_providers(topic.clone(), key.clone()).await?;

                    yield CommandOutput::spinner("Waiting for Providers...", &["◐", "◑", "◒", "◓"]);

                    while let Some(event) = providers_stream.next().await {

                        match event {
                            Ok(provider) => yield CommandOutput::result("kv/get-providers")
                                .with_field("topic", OutputField::String(topic.clone()))
                                .with_field("key", OutputField::String(key.clone()))
                                .with_field("provider", OutputField::PeerId(provider))
                                .with_human_readable_template("🤖 {provider}"),
                            Err(e) => yield CommandOutput::error("kv/get-providers", &e.to_string())
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
