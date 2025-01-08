use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::inspect::Inspect;
use crate::boxed_try_stream;
use hyveos_core::debug::MessageDebugEventType;
use crate::error::HyveCtlResult;

impl CommandFamily for Inspect {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut debug = connection.debug();

        match self {
            Inspect::Mesh { .. } => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_mesh_topology().await?;

                    yield CommandOutput::spinner("Waiting for Topology Events...", &["◐", "◒", "◑", "◓"]);

                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(message) => {
                                yield CommandOutput::result("inspect/mesh")
                                    .with_field("event", OutputField::MeshTopologyEvent(message))
                                    .with_human_readable_template("📡 {event}")
                            },
                            Err(e) => {
                                yield CommandOutput::error("inspect/mesh", &e.to_string())
                            }
                        }
                    }
                }
            },
            Inspect::Services => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_messages().await?;

                    yield CommandOutput::spinner("Waiting for Service Events...", &["◐", "◑", "◒", "◓"]);

                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(message) => {
                                let mut output = CommandOutput::result("inspect/services");

                                output = match message.event {
                                    MessageDebugEventType::Request(request) => {
                                        output
                                        .with_field("service", OutputField::String(String::from("reqres/req")))
                                        .with_field("id", OutputField::String(request.id.to_string()))
                                        .with_field("receiver", OutputField::PeerId(request.receiver))
                                        .with_field("request", OutputField::Request(request.msg))
                                        .with_human_readable_template("💬 {request} received by {receiver} under {id}")
                                    }
                                    MessageDebugEventType::Response(response) => {
                                        output
                                        .with_field("service", OutputField::String(String::from("reqres/res")))
                                        .with_field("req_id", OutputField::String(response.req_id.to_string()))
                                        .with_field("response", OutputField::Response(response.response))
                                        .with_human_readable_template("🗨️ {response} sent for {req_id}")
                                    }
                                    MessageDebugEventType::GossipSub(message) => {
                                        output
                                        .with_field("service", OutputField::String(String::from("pub-sub")))
                                        .with_field("message", OutputField::GossipMessage(message))
                                        .with_human_readable_template("📨 {message}")
                                    }
                                };

                                yield output
                            },
                            Err(e) => {
                                yield CommandOutput::error("inspect/services", &e.to_string())
                            }
                        }
                    }
                }
            }
        }
    }
}
