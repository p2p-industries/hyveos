use std::{sync::Arc, time::Duration};

use macaddress::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatmanNeighbour {
    pub if_name: String,
    pub last_seen: Duration,
    pub mac: MacAddress,
    pub throughput_kbps: Option<u32>,
}

#[tarpc::service]
pub trait BatmanNeighboursServer {
    async fn get_neighbours(if_name: Arc<str>) -> Result<Vec<BatmanNeighbour>, String>;
}
