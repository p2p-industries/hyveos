use futures::stream::BoxStream;
use hyvectl_commands::families::hyve::Hyve;
use hyveos_sdk::{services::ScriptingConfig, Connection, PeerId};
use ulid::Ulid;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for Hyve {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut scripting_service = connection.scripting();

        match self {
            Hyve::Start {
                image,
                peer,
                ports,
                persistent,
                local,
            } => {
                boxed_try_stream! {
                    let mut config = ScriptingConfig::new(&image);

                    config = if let Some(peer) = peer.clone() { config.target(peer.parse()?) }
                    else { config };

                    config = if local { config.local() } else { config };

                    config = if persistent { config.persistent() } else { config };

                    for port in ports {
                        config = config.expose_port(port);
                    }

                    yield CommandOutput::spinner("Deploying image...", &["◐", "◑", "◒", "◓"]);

                    scripting_service.deploy_script(config).await?;

                    yield CommandOutput::result()
                        .with_field("image", image)
                        .with_field("peer", peer.unwrap_or("local".to_string()))
                        .with_tty_template("Deployed { {image} } on { {peer} }")
                        .with_non_tty_template("{image}")
                }
            }
            Hyve::List { peer } => {
                boxed_try_stream! {
                    let peer_parsed = match peer {
                        Some (p) => Some(p.parse::<PeerId>()?),
                        None => None
                    };

                    let scripts = scripting_service.list_running_scripts(peer_parsed).await?;

                    for script in scripts {
                        let mut out = CommandOutput::result()
                            .with_field("image", script.image.to_string())
                            .with_field("id", script.id.to_string());

                        out = match script.name {
                            Some(name) => {
                                out.with_field("name", name.to_string())
                                .with_tty_template("💾 { name: {name}, image: {image}, id: {id} }")
                                .with_non_tty_template("{name},{image},{id}")
                            }
                            None => {out.with_tty_template("💾 { image: {image}, id: {id} }")
                                        .with_non_tty_template("{image},{id}")}
                        };

                        yield out;
                    }
                }
            }
            Hyve::Stop { peer, id } => {
                boxed_try_stream! {

                    let peer_parsed = match peer.clone() {
                        Some (p) => Some(p.parse::<PeerId>()?),
                        None => None
                    };

                    scripting_service.stop_script(id.parse::<Ulid>()?, peer_parsed).await?;

                    yield CommandOutput::result()
                            .with_field("peer", peer.unwrap_or("local".to_string()))
                            .with_field("id", id)
                            .with_tty_template("Stopped { {id} } on { {peer} }")
                            .with_non_tty_template("{peer},{id}")
                }
            }
        }
    }
}
