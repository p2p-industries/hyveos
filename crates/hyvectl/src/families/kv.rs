use futures::{stream::BoxStream, StreamExt};
use hyvectl_commands::families::kv::Kv;
use hyveos_sdk::Connection;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};
impl CommandFamily for Kv {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut dht = connection.dht();

        match self {
            Kv::Get { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Retrieved { {value} } under { {key} } in topic { {topic} }"},
                        None => {"🔑 Retrieved { {value} } under { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    let result = dht.get_record(&topic, key.clone()).await?;

                    match result {
                        Some(res) => yield CommandOutput::result()
                            .with_field("topic", topic)
                            .with_field("key", key)
                            .with_field("value", String::from_utf8(res)?.into())
                            .with_tty_template(template)
                            .with_non_tty_template("{value}"),
                        None => yield CommandOutput::result()
                            .with_field("topic", topic)
                            .with_field("key", key)
                            .with_tty_template(template)
                            .with_non_tty_template("Unable to retrieve key {key} in {topic}"),
                    }
                }
            }
            Kv::Put { key, value, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Added { {value} } under { {key} } in topic { {topic} }"},
                        None => {"🔑 Added { {value} } under { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    dht.put_record(&topic, key.clone(), value.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("key", key.clone())
                        .with_field("value", value.clone())
                        .with_tty_template(template)
                        .with_non_tty_template("{value},{key},{topic}");
                }
            }
            Kv::Provide { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Now providing { {key} } in topic { {topic} }"},
                        None => {"🔑 Now providing { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();


                    dht.provide(&topic, key.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("key", key)
                        .with_tty_template(template)
                        .with_non_tty_template("{key},{topic}")
                }
            }
            Kv::GetProviders { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    let mut providers_stream = dht.get_providers(topic.clone(), key.clone()).await?;

                    yield CommandOutput::spinner("Fetching Providers...", &["◐", "◑", "◒", "◓"]);

                    while let Some(event) = providers_stream.next().await {

                        match event {
                            Ok(provider) => yield CommandOutput::result()
                                .with_field("topic", topic.clone())
                                .with_field("key", key.clone())
                                .with_field("provider", provider.to_string())
                                .with_tty_template("🤖 { {provider} }")
                                .with_non_tty_template("{provider}"),
                            Err(e) => Err(e)?
                        }
                    }
                }
            }
            Kv::StopProvide { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Stopped providing { {key} } in topic { {topic} }"},
                        None => {"🔑 Stopped providing { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    dht.stop_providing(&topic, key.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("key", key)
                        .with_tty_template(template)
                        .with_non_tty_template("{key},{topic}")
                }
            }
        }
    }
}
