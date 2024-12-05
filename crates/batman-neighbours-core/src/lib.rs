#![warn(clippy::expect_used, clippy::unwrap_used, clippy::pedantic)]

use std::time::Duration;

use hyveos_macaddress::MacAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatmanNeighbour {
    pub if_index: u32,
    pub last_seen: Duration,
    pub mac: MacAddress,
    pub throughput_kbps: Option<u32>,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum Error {
    #[error("Failed to send netlink request: {0}")]
    FailedToSendRequest(String),
    #[error("Failed to decode netlink response: {0}")]
    FailedToDecodeResponse(String),
    #[error("Expected response message not request")]
    ExpectedResponseMessage,
    #[error("Received netlink error: {0}")]
    NetlinkError(String),
}

#[tarpc::service]
pub trait BatmanNeighboursServer {
    async fn get_neighbours(if_index: u32) -> Result<Vec<BatmanNeighbour>, Error>;
}
