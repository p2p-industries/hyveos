use hyveos_sdk::Connection;
use std::error::Error;
use crate::util::{resolve_stream, CommandFamily, DynError};
use crate::output::{CommandOutput, OutputField};
use futures::{StreamExt, TryStreamExt, FutureExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::inspect::Inspect;


impl CommandFamily for Inspect {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut debug = connection.debug();

        match self {
            Inspect::Mesh { local } => {
              let mesh_stream = resolve_stream(debug.subscribe_mesh_topology()
                  .await).await;

                mesh_stream
                    .map_ok(move |event| {
                        CommandOutput::new_result("Inspect Mesh")
                            .with_field("event", OutputField::MeshTopologyEvent(event))
                            .with_human_readable_template("Mesh Topology changed: {event}")
                    }).map_err(|e| e.into())
                    .boxed()

            },
            Inspect::Services => {
                let debug_stream = resolve_stream(debug.subscribe_messages()
                    .await).await;

                debug_stream
                    .map_ok(move |event| {
                        CommandOutput::new_result("Inspect Services")
                            .with_field("event", OutputField::ServiceDebugEvent(event))
                            .with_human_readable_template("Service event: {event}")
                    }).map_err(|e| e.into())
                    .boxed()
            }
        }
    }
}
