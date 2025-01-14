use futures::{stream::BoxStream, TryStreamExt as _};
use hyvectl_commands::families::inspect::Inspect;
use hyveos_core::{
    debug::{MessageDebugEvent, MessageDebugEventType},
    discovery::NeighbourEvent,
};
use hyveos_sdk::Connection;

use crate::{
    boxed_try_stream,
    error::{HyveCtlError, HyveCtlResult},
    out::CommandOutput,
    util::CommandFamily,
};

impl TryFrom<MessageDebugEvent> for CommandOutput {
    type Error = HyveCtlError;

    fn try_from(event: MessageDebugEvent) -> Result<Self, Self::Error> {
        let out = CommandOutput::result();

        Ok(match event.event {
            MessageDebugEventType::Request(req) => out
                .with_field("service", "req-res/request".to_string())
                .with_field("receiver", req.receiver.to_string())
                .with_field("id", req.id.to_string())
                .with_field("topic", req.msg.topic.unwrap_or_default())
                .with_field("data", String::from_utf8(req.msg.data)?)
                .with_tty_template("💬 {{ receiver: {receiver}, id: {id}, data: {data} }}")
                .with_non_tty_template("{service},{receiver},{id},{topic},{data}"),
            MessageDebugEventType::Response(res) => out
                .with_field("service", "resp-res/response".to_string())
                .with_field("id", res.req_id.to_string())
                .with_field("data", String::from_utf8(res.response.try_into()?)?)
                .with_tty_template("🗨️ {{ id: {id}, data: {data} }}")
                .with_non_tty_template("{service},{id},{data}"),
            MessageDebugEventType::GossipSub(msg) => out
                .with_field("service", "pub-sub".to_string())
                .with_field("topic", msg.topic.to_string())
                .with_field("data", String::from_utf8(msg.data)?)
                .with_tty_template("📨 {{ topic: {topic}, data: {data} }}")
                .with_non_tty_template("{service},{topic},{data}"),
        })
    }
}

impl CommandFamily for Inspect {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut debug = connection.debug();

        match self {
            Inspect::Mesh { .. } => {
                boxed_try_stream! {
                    yield CommandOutput::spinner("Waiting for Topology Events...", &["◐", "◒", "◑", "◓"]);

                    let mut stream = debug.subscribe_mesh_topology().await?;

                    while let Some(event) = stream.try_next().await? {
                        let out = CommandOutput::result()
                            .with_field("source", event.peer_id.to_string());

                        match event.event {
                            NeighbourEvent::Init(peers) => {
                                for peer in peers {
                                    yield out.clone()
                                        .with_field("type", "connected".to_string())
                                        .with_field("peer", peer.to_string())
                                        .with_tty_template("📡 Connected { {peer} } to { {source} }")
                                        .with_non_tty_template("{peer},{source}")
                                }
                            },
                            NeighbourEvent::Discovered(peer) => {
                                yield out.with_field("type", "discovered".to_string())
                                    .with_field("peer", peer.to_string())
                                    .with_tty_template("📡 Discovered { {peer} } from { {source} }")
                                    .with_non_tty_template("{peer},{source}")
                            },
                            NeighbourEvent::Lost(peer) => {
                                yield out.with_field("type", "lost".to_string())
                                    .with_field("peer", peer.to_string())
                                    .with_tty_template("📡 Lost { {peer} } from { {source} }")
                                    .with_non_tty_template("{peer},{source}")
                            }
                        }
                    }
                }
            }
            Inspect::Services => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_messages().await?;

                    yield CommandOutput::spinner("Waiting for Service Events...", &["◐", "◑", "◒", "◓"]);

                    while let Some(event) = stream.try_next().await? {
                        yield event.try_into()?
                    }
                }
            }
        }
    }
}
