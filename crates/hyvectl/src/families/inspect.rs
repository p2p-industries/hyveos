use hyveos_sdk::Connection;
use crate::util::{CommandFamily};
use crate::output::{CommandOutput};
use futures::{StreamExt};
use futures::stream::BoxStream;
use hyvectl_commands::families::inspect::Inspect;
use crate::boxed_try_stream;
use hyveos_core::debug::{MeshTopologyEvent, MessageDebugEvent, MessageDebugEventType};
use hyveos_core::discovery::NeighbourEvent;
use hyveos_core::req_resp::Response;
use crate::error::{HyveCtlError, HyveCtlResult};

impl From<MeshTopologyEvent> for CommandOutput {
    fn from(event: MeshTopologyEvent) -> Self {
        let mut out = CommandOutput::result("inspect/mesh");

        out =  match event.event {
            NeighbourEvent::Init(peers) => {
                out.with_field("type", "connected".to_string().into())
                    .with_field("peers", peers.into())
                    .with_tty_template("📡 Connected to {{peers}}")
            },
            NeighbourEvent::Discovered(peer) => {
                out.with_field("type", "discovered".to_string().into())
                    .with_field("peer", peer.into())

                    .with_tty_template("📡 Discovered {{peer}}")
            },
            NeighbourEvent::Lost(peer) => {
                out.with_field("type", "lost".to_string().into())
                    .with_field("peer", peer.into())
                    .with_tty_template("📡 Lost {{peer}}")
            }
        };

        out.with_non_tty_template("{peers}")
    }
}

impl TryFrom<MessageDebugEvent> for CommandOutput {
    type Error = HyveCtlError;

    fn try_from(event: MessageDebugEvent) -> Result<Self, Self::Error> {
        let mut out = CommandOutput::result("inspect/services");

        out = match event.event {
            MessageDebugEventType::Request(req) => {
                out.with_field("service", "req-res/request".to_string().into())
                    .with_field("receiver", req.receiver.into())
                    .with_field("id", req.id.to_string().into())
                    .with_field("topic", req.msg.topic.unwrap_or_default().into())
                    .with_field("data", String::from_utf8(req.msg.data)?.into())
                    .with_tty_template("💬 {{ receiver: {receiver}, id: {id}, data: {data} }}")
                    .with_non_tty_template("{service},{receiver},{id},{topic},{data}")
            }
            MessageDebugEventType::Response(res) => {
                out = out.with_field("service", "resp-res/response".to_string().into())
                    .with_field("id", res.req_id.to_string().into());

                match res.response {
                    Response::Data(data) => {
                        out.with_field("data", String::from_utf8(data)?.into())
                            .with_tty_template("🗨️ {{ id: {id}, data: {data} }}")
                            .with_non_tty_template("{service},{id},{data}")
                    }
                    Response::Error(e) => {Err(e)?}
                }
            }
            MessageDebugEventType::GossipSub(msg) => {
                out.with_field("service", "pub-sub".to_string().into())
                    .with_field("topic", msg.topic.to_string().into())
                    .with_field("data", String::from_utf8(msg.data)?.into())
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
                        yield event?.into()
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
