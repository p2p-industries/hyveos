use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::out::{CommandOutput};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::inspect::Inspect;
use crate::boxed_try_stream;
use hyveos_core::debug::{MessageDebugEvent, MessageDebugEventType};
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::req_resp::Response;
use crate::error::{HyveCtlError, HyveCtlResult};

impl TryFrom<MessageDebugEvent> for CommandOutput {
    type Error = HyveCtlError;

    fn try_from(event: MessageDebugEvent) -> Result<Self, Self::Error> {
        let mut out = CommandOutput::result();

        out = match event.event {
            MessageDebugEventType::Request(req) => {
                out.with_field("service", "req-res/request".to_string())
                    .with_field("receiver", req.receiver.to_string())
                    .with_field("id", req.id.to_string())
                    .with_field("topic", req.msg.topic.unwrap_or_default())
                    .with_field("data", String::from_utf8(req.msg.data)?)
                    .with_tty_template("💬 {{ receiver: {receiver}, id: {id}, data: {data} }}")
                    .with_non_tty_template("{service},{receiver},{id},{topic},{data}")
            }
            MessageDebugEventType::Response(res) => {
                out = out.with_field("service", "resp-res/response".to_string())
                    .with_field("id", res.req_id.to_string());

                match res.response {
                    Response::Data(data) => {
                        out.with_field("data", String::from_utf8(data)?)
                            .with_tty_template("🗨️ {{ id: {id}, data: {data} }}")
                            .with_non_tty_template("{service},{id},{data}")
                    }
                    Response::Error(e) => {Err(e)?}
                }
            }
            MessageDebugEventType::GossipSub(msg) => {
                out.with_field("service", "pub-sub".to_string())
                    .with_field("topic", msg.topic.to_string())
                    .with_field("data", String::from_utf8(msg.data)?)
                    .with_tty_template("📨 {{ topic: {topic}, data: {data} }}")
                    .with_non_tty_template("{service},{topic},{data}")
            }
        };

        Ok(out)
    }
}

impl CommandFamily for Inspect {
    async fn run(self, connection: &Connection) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut debug = connection.debug();

        match self {
            Inspect::Mesh { .. } => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_mesh_topology().await?;

                    yield CommandOutput::spinner("Waiting for Topology Events...", &["◐", "◒", "◑", "◓"]);

                    while let Some(event) = stream.next().await {
                        let event = event?;

                        let out = CommandOutput::result()
                        .with_field("source", event.peer_id.to_string());

                        match event.event {
                            NeighbourEvent::Init(peers) => {
                                for peer in peers {
                                    yield out.clone().with_field("type", "connected".to_string())
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
            },
            Inspect::Services => {
                boxed_try_stream! {
                    let mut stream = debug.subscribe_messages().await?;

                    yield CommandOutput::spinner("Waiting for Service Events...", &["◐", "◑", "◒", "◓"]);

                    while let Some(event) = stream.next().await {
                        yield event?.try_into()?
                    }
                }
            }
        }
    }
}
