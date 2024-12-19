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
                    .send_request(peer.parse().unwrap(), message, topic).await?;
                stream::once(async move {
                    CommandOutput::new("ReqRes Send")
                        .add_field("response", OutputField::Response(response))
                        .with_human_readable_template("Got response {response}")
                }).boxed()
            }
            ReqRes::Receive {} => {
                let req_stream = resolve_stream(reqres.recv(None).await).await;

                req_stream
                    .map_ok(move |req| {
                        CommandOutput::new("ReqRes Receive")
                            .add_field("request", OutputField::Request(req.0))
                            .with_human_readable_template("Retrieved request {request}")
                    }).map_err(|e| e.into())
                    .boxed()
            }
            ReqRes::Respond { id, message } => {
                todo!()
            }
        }
    }
}