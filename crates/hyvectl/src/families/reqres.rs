use crate::boxed_try_stream;
use crate::error::HyveCtlResult;
use crate::out::CommandOutput;
use crate::util::CommandFamily;
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use hyvectl_commands::families::reqres::ReqRes;
use hyveos_core::req_resp::Response;
use hyveos_sdk::Connection;
use hyveos_sdk::PeerId;

impl CommandFamily for ReqRes {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut reqres = connection.req_resp();

        match self {
            ReqRes::Receive {} => {
                boxed_try_stream! {
                    yield CommandOutput::spinner("Waiting for Requests...", &["◐", "◒", "◑", "◓"]);

                    while let Some(request) = reqres.recv(None).await?.try_next().await? {
                        yield CommandOutput::result()
                        .with_field("peer_id", request.0.peer_id.to_string())
                        .with_field("topic", request.0.topic.unwrap_or_default())
                        .with_field("data", String::from_utf8(request.0.data)?.into())
                        .with_field("id", request.1.id().to_string())
                        .with_tty_template("💬 [ID: {id}] { peer: {peer_id}, topic: {topic}, data: {data} }")
                        .with_non_tty_template("{id},{peer_id},{topic},{data}");
                    }
                }
            }
            ReqRes::Send {
                peer,
                request: message,
                topic,
            } => {
                boxed_try_stream! {
                    let peer_id = peer.parse::<PeerId>()?;

                    yield CommandOutput::spinner("Waiting for Response", &["◐", "◒", "◑", "◓"]);

                    let response = reqres.send_request(peer_id, message, topic).await?;

                    let mut output = CommandOutput::result();

                    output = match response {
                        Response::Data(data) => {
                            output
                                .with_field("response", String::from_utf8(data)?.into())
                        },
                        Response::Error(e) => {
                            Err(e)?
                        }
                    };

                    yield output
                    .with_tty_template("🗨  {response}")
                    .with_non_tty_template("{response}")
                }
            }
            ReqRes::Respond {
                id,
                response: message,
            } => {
                boxed_try_stream! {
                    reqres.respond(id, Response::Data(message.clone().into())).await?;

                    yield CommandOutput::result()
                    .with_field("id", id.to_string())
                    .with_field("response", message)
                    .with_tty_template("Sent { {response} } for { {id} }")
                    .with_non_tty_template("{id},{response}")
                }
            }
        }
    }
}
