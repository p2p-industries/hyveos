use futures::stream::BoxStream;
use hyvectl_commands::families::kv::Kv;
use hyveos_sdk::Connection;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for Kv {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut kv_service = connection.kv();

        match self {
            Kv::Get { key, topic } => {
                boxed_try_stream! {
                    let template = match topic {
                        Some(_) => {"🔑 Retrieved { {value} } under { {key} } in topic { {topic} }"},
                        None => {"🔑 Retrieved { {value} } under { {key} }"}
                    };

                    let topic = topic.unwrap_or_default();

                    let result = kv_service.get_record(&topic, key.clone()).await?;

                    match result {
                        Some(res) => yield CommandOutput::result()
                            .with_field("topic", topic)
                            .with_field("key", key)
                            .with_field("value", String::from_utf8(res)?)
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

                    kv_service.put_record(&topic, key.clone(), value.clone()).await?;

                    yield CommandOutput::result()
                        .with_field("topic", topic)
                        .with_field("key", key.clone())
                        .with_field("value", value.clone())
                        .with_tty_template(template)
                        .with_non_tty_template("{value},{key},{topic}");
                }
            }
        }
    }
}
