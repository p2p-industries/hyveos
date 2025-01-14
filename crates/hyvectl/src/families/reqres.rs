use futures::{stream::BoxStream, TryStreamExt as _};
use hyvectl_commands::families::reqres::ReqRes;
use hyveos_core::req_resp::{Response, TopicQuery};
use hyveos_sdk::{Connection, PeerId};

use crate::{boxed_try_stream, error::HyveCtlResult, out::CommandOutput, util::CommandFamily};

impl CommandFamily for ReqRes {
    async fn run(
        self,
        connection: &Connection,
    ) -> BoxStream<'static, HyveCtlResult<CommandOutput>> {
        let mut req_res_service = connection.req_resp();

        match self {
            ReqRes::Receive { topic } => {
                boxed_try_stream! {
                    yield CommandOutput::spinner("Waiting for Requests...", &["◐", "◒", "◑", "◓"]);

                    let query = topic.map(Into::into).map(TopicQuery::String);

                    let mut stream = req_res_service.recv(query).await?;

                    while let Some(request) = stream.try_next().await? {
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

                    let response = req_res_service.send_request(peer_id, message, topic).await?;

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
                    req_res_service.respond(id, Response::Data(message.clone().into())).await?;

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
