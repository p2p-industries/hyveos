use std::error::Error;
use futures::stream::BoxStream;
use hyvectl_commands::families::reqres::ReqRes;
use hyveos_sdk::Connection;
use crate::output::{CommandOutput, OutputField};
use crate::util::{resolve_stream, CommandFamily, DynError};
use futures::{StreamExt, stream, TryStreamExt, FutureExt};

impl CommandFamily for ReqRes {
    async fn run(self, connection: &Connection) -> BoxStream<'static, Result<CommandOutput, DynError>> {
        let mut reqres = connection.req_resp();

        match self {
            ReqRes::Send { peer, message, topic } => {
                todo!()
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