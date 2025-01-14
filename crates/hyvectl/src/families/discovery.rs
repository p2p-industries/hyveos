use futures::{stream::BoxStream, TryStreamExt as _};
use hyvectl_commands::families::discovery::Discovery;
use hyveos_sdk::Connection;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for Discovery {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut discovery_service = connection.discovery();

        match self {
            Discovery::Provide { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Now providing { {key} } in topic { {topic} }"},
                        None => {"🔑 Now providing { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    discovery_service.provide(&topic, key.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("key", key)
                        .with_tty_template(template)
                        .with_non_tty_template("{key},{topic}")
                }
            }
            Discovery::GetProviders { key, topic } => {
                boxed_try_stream! {
                    let topic = topic.clone().unwrap_or_default();

                    yield CommandOutput::spinner("Fetching Providers...", &["◐", "◑", "◒", "◓"]);

                    let mut providers_stream = discovery_service.get_providers(topic.clone(), key.clone()).await?;

                    while let Some(provider) = providers_stream.try_next().await? {
                        yield CommandOutput::result()
                            .with_field("topic", topic.clone())
                            .with_field("key", key.clone())
                            .with_field("provider", provider.to_string())
                            .with_tty_template("🤖 { {provider} }")
                            .with_non_tty_template("{provider}");
                    }
                }
            }
            Discovery::StopProvide { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Stopped providing { {key} } in topic { {topic} }"},
                        None => {"🔑 Stopped providing { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    discovery_service.stop_providing(&topic, key.clone()).await?;

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
