use std::error::Error;
use futures::stream::BoxStream;
use hyvectl_commands::families::reqres::ReqRes;
use hyveos_sdk::Connection;
use crate::output::{CommandOutput, OutputField};
use crate::util::{resolve_stream, CommandFamily};
use futures::{StreamExt, stream, TryStreamExt, FutureExt};

impl CommandFamily for ReqRes {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>> {
        let mut reqres = connection.req_resp();

        match self {
            ReqRes::Send { peer, message, topic } => {
                let response = reqres
                    .send_request(peer.parse().unwrap(), message, topic).await.unwrap();
                stream::once(async move {
                    Ok(CommandOutput::new("ReqRes Send")
                        .add_field("response", OutputField::Response(response))
                        .with_human_readable_template("Got response {response}"))
                }).boxed()
            }
            ReqRes::Receive {} => {
               todo!()
            }
            ReqRes::Respond { id, message } => {
                todo!()
            }
        }
    }
}