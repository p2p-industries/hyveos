use futures::{stream, StreamExt};
use futures::stream::BoxStream;
use crate::util::{CommandFamily, DynError};
use hyvectl_commands::families::hyve::Hyve;
use hyveos_sdk::{Connection, PeerId};
use hyveos_sdk::services::ScriptingConfig;
use crate::output::{CommandOutput, OutputField};
use crate::boxed_try_stream;
use ulid::Ulid;

impl CommandFamily for Hyve {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut scripting_service = connection.scripting();

        match self {
            Hyve::Start {image, peer, local, ports} => {
                boxed_try_stream! {
                    let mut config = ScriptingConfig::new(&image);

                    config = if let Some(peer) = peer.clone(){config.target(peer.clone().parse()?)}
                    else {config.local()};

                    for port in ports {
                        config = config.expose_port(port)
                    }

                    yield CommandOutput::spinner("Deploying image...", &["◐", "◑", "◒", "◓"]);

                    scripting_service.deploy_script(config).await?;

                    yield CommandOutput::result("hyve/start")
                        .with_field("image", OutputField::String(image))
                        .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                        .with_human_readable_template("Deployed {image} on {peer}")
                }
            },
            Hyve::List {peer, local } => {
                boxed_try_stream! {
                    let peer_parsed = match peer.clone() {
                        Some (p) => Some(p.parse::<PeerId>()?),
                        None => None
                    };

                    let scripts = scripting_service.list_running_scripts(peer_parsed).await?;

                    yield CommandOutput::result("hyve/list")
                            .with_field("scripts", OutputField::RunningScripts(scripts))
                            .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                            .with_human_readable_template("Running scripts on {peer} : {scripts}")
                }
            },
            Hyve::Stop {peer, local, id} => {
                boxed_try_stream! {

                    let peer_parsed = match peer.clone() {
                        Some (p) => Some(p.parse::<PeerId>()?),
                        None => None
                    };

                    scripting_service.stop_script(id.parse::<Ulid>()?, peer_parsed).await?;

                    yield CommandOutput::result("hyve/stop")
                            .with_field("peer", OutputField::String(peer.unwrap_or("local".to_string())))
                            .with_field("id", OutputField::String(id))
                            .with_human_readable_template("Stopped {id} on {peer}")
                }
            }
        }
    }
}