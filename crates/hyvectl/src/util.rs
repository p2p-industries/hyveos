use p2p_industries_sdk::P2PConnection;
use async_trait::async_trait;
use std::error::Error;
use futures::stream::BoxStream;
use crate::output::CommandOutput;

#[async_trait]
pub trait CommandFamily {
    async fn run(self, connection: &P2PConnection) -> BoxStream<'static, Result<CommandOutput, Box<dyn Error>>>;
}
