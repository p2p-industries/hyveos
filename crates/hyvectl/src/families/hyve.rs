use std::error::Error;
use clap::builder::TypedValueParser;
use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use crate::util::{CommandFamily, DynError};
use hyvectl_commands::families::hyve::Hyve;
use hyveos_sdk::Connection;
use hyveos_sdk::services::ScriptingConfig;
use crate::output::{CommandOutput, OutputField};
use crate::single_output_stream;

impl CommandFamily for Hyve {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut scripting_service = connection.scripting();

        match self {
            Hyve::Start {image, peer, local, ports} => {
                let mut config = ScriptingConfig::new(&image);

                config = if let Some(peer) = peer.clone() {config.target(peer.clone().parse().unwrap())}
                else {config.local()};

                for port in ports {
                    config = config.expose_port(port)
                }

                scripting_service.deploy_script(config).await.unwrap();
                single_output_stream!(
                    CommandOutput::new_result("Hyve Start")
                        .with_field("image", OutputField::String(image))
                        .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                        .with_human_readable_template("Deployed {image} on {peer}")
                )
            },
            Hyve::List {peer, local } => {
                let scripts = scripting_service.list_running_scripts(peer.clone()
                    .map(|p| p.parse().unwrap())).await.unwrap();

                single_output_stream!(CommandOutput::new_result("Hyve List")
                .with_field("scripts", OutputField::RunningScripts(scripts))
                .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                .with_human_readable_template("Running scripts on {peer} : {scripts}"))
            },
            Hyve::Stop {peer, local,id} => {
                scripting_service.stop_script(id.parse().unwrap(), peer.clone()
                    .map(|p| p.parse().unwrap())).await.unwrap();

                single_output_stream!(CommandOutput::new_result("Hyve Stop")
                    .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                    .with_field("id", OutputField::String(id))
                .with_human_readable_template("Stopped {id} on {peer}"))
            }
        }
    }
}