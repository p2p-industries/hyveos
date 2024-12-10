use p2p_industries_sdk::P2PConnection;
use clap::Subcommand;
use async_trait::async_trait;
use std::error::Error;
use crate::util::CommandFamily;
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, stream};
use futures::stream::BoxStream;

#[derive(Subcommand)]
pub enum Kv {
    #[command(about = "Publish the value for the given key")]
    Put {
        key: String,
        value: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the value for the given key")]
    Get {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Announce that this node can provide the value for the given key")]
    Provide {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Get the providers for the given key")]
    GetProviders {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    },
    #[command(about = "Stop providing a given key")]
    StopProvide {
        key: String,
        #[arg(long)]
        topic: Option<String>,
    }
}

#[async_trait]
impl CommandFamily for Kv {
    async fn run(self, connection: &P2PConnection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        let mut dht = connection.dht();

        match self {
            Kv::Put { key, value, topic } => {
                let topic = topic.unwrap_or_default();
                stream::once(async move {
                    dht.put_record(key.clone(), value.clone(), topic.clone()).await?;
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
                    let result = dht.get_record(key.clone(), topic.clone()).await?;
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
                    dht.provide(key.clone(), topic.clone()).await?;
                    Ok(CommandOutput::new("KV Provide")
                        .add_field("key", OutputField::String(key))
                        .add_field("topic", OutputField::String(topic))
                        .with_human_readable_template("Started providing key {key} in topic {topic}")
                    )
                }).boxed()
            },
            Kv::GetProviders { key, topic } => {
                let topic = topic.unwrap_or_default();
                let providers_future = dht.get_providers(key.clone(), topic.clone());
                let providers_stream = match providers_future.await {
                    Ok(s) => s,
                    Err(e) => {
                        return stream::once(async move { Err(e.into()) }).boxed();
                    }
                };

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
                unreachable!()
            }
        }
    }
}
