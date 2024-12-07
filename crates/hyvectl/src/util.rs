use p2p_industries_sdk::P2PConnection;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait CommandFamily {
    async fn run(self, connection: &P2PConnection) -> Result<(), Box<dyn Error>>;
}
