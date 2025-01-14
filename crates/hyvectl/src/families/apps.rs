use std::str::FromStr;

use futures::stream::BoxStream;
use hyvectl_commands::families::apps::Apps;
use hyveos_sdk::{services::AppConfig, Connection, PeerId};
use ulid::Ulid;

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for Apps {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut apps_service = connection.apps();

        match self {
            Apps::Start {
                image,
                peer,
                ports,
                persistent,
                local,
            } => {
                boxed_try_stream! {
                    let mut config = AppConfig::new(&image);

                    if let Some(peer) = peer.clone() {
                        config = config.target(peer.parse()?);
                    }

                    if local {
                        config = config.local();
                    }

                    if persistent {
                        config = config.persistent();
                    }

                    for port in ports {
                        config = config.expose_port(port);
                    }

                    yield CommandOutput::spinner("Deploying app...", &["◐", "◑", "◒", "◓"]);

                    apps_service.deploy(config).await?;

                    yield CommandOutput::result()
                        .with_field("image", image)
                        .with_field("peer", peer.unwrap_or("local".to_string()))
                        .with_tty_template("Deployed { {image} } on { {peer} }")
                        .with_non_tty_template("{image}")
                }
            }
            Apps::List { peer } => {
                boxed_try_stream! {
                    let peer_id = peer.as_deref().map(PeerId::from_str).transpose()?;

                    let apps = apps_service.list_running(peer_id).await?;

                    for app in apps {
                        let out = CommandOutput::result()
                            .with_field("image", app.image.to_string())
                            .with_field("id", app.id.to_string());

                        yield match app.name {
                            Some(name) => out
                                .with_field("name", name.to_string())
                                .with_tty_template("💾 { name: {name}, image: {image}, id: {id} }")
                                .with_non_tty_template("{name},{image},{id}"),
                            None => out
                                .with_tty_template("💾 { image: {image}, id: {id} }")
                                .with_non_tty_template("{image},{id}"),
                        };
                    }
                }
            }
            Apps::Stop { peer, id } => {
                boxed_try_stream! {
                    let peer_id = peer.as_deref().map(PeerId::from_str).transpose()?;

                    apps_service.stop(id.parse::<Ulid>()?, peer_id).await?;

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
