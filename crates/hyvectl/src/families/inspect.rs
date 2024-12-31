use hyveos_sdk::Connection;
use crate::util::{CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, FutureExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::inspect::Inspect;
use crate::boxed_try_stream;

impl CommandFamily for Inspect {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut debug = connection.debug();

        match self {
            Inspect::Mesh { local } => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_mesh_topology().await?;

                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(message) => {
                                yield CommandOutput::result("Inspect Mesh")
                                    .with_field("event", OutputField::MeshTopologyEvent(message))
                                    .with_human_readable_template("Mesh Topology changed: {event}")
                            },
                            Err(e) => {
                                yield CommandOutput::error("Inspect Mesh", &e.to_string())
                            }
                        }
                    }
                }
            },
            Inspect::Services => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_messages().await?;

                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(message) => {
                                yield CommandOutput::result("Inspect Services")
                                    .with_field("event", OutputField::ServiceDebugEvent(message))
                                    .with_human_readable_template("Service event: {event}")
                            },
                            Err(e) => {
                                yield CommandOutput::error("Inspect Services", &e.to_string())
                            }
                        }
                    }
                }
            }
        }
    }
}
