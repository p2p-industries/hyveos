use futures::stream::BoxStream;
use hyvectl_commands::families::reqres::ReqRes;
use hyveos_sdk::Connection;
use crate::boxed_try_stream;
use crate::output::{CommandOutput, OutputField};
use crate::util::{CommandFamily, DynError};
use hyveos_sdk::PeerId;
use futures::{StreamExt, TryStreamExt};
use hyveos_core::req_resp::Response;

impl CommandFamily for ReqRes {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut reqres = connection.req_resp();

        match self {
            ReqRes::Send { peer, request: message, topic } => {
                boxed_try_stream! {
                    let peer_id = peer.parse::<PeerId>()?;

                    yield CommandOutput::spinner("Waiting for Response", &["◐", "◒", "◑", "◓"]);

                    let response = reqres.send_request(peer_id, message.clone(), topic.clone()).await?;

                    yield CommandOutput::result("reqres/req")
                    .with_field("from", OutputField::PeerId(peer_id))
                    .with_field("response", OutputField::Response(response))
                    .with_human_readable_template("🗨  {from}: {response}")
                }
            }
            ReqRes::Receive {} => {
               boxed_try_stream! {
                   yield CommandOutput::spinner("Waiting for Requests...", &["◐", "◒", "◑", "◓"]);

                   while let Some(request) = reqres.recv(None).await?.try_next().await? {
                       yield CommandOutput::result("reqres/recv")
                       .with_field("request", OutputField::InboundRequest(request.0))
                       .with_field("id", OutputField::String(request.1.id().to_string()))
                       .with_human_readable_template("💬 [ID: {id}] {request}");
                   }
               }
            }
            ReqRes::Respond { id, response: message } => {
                boxed_try_stream! {
                    reqres.respond(id, Response::Data(message.clone().into())).await?;

                    yield CommandOutput::result("reqres/res")
                    .with_field("id", OutputField::String(id.to_string()))
                    .with_field("response", OutputField::String(message))
                    .with_human_readable_template("Sent {response} for {id}")
                }
            }
        }
    }
}