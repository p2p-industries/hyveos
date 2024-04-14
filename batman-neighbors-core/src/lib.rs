use macaddress::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatmanNeighbor {
    pub if_name: String,
    pub last_seen_msecs: u32,
    pub mac: MacAddress,
    pub throughput_kbps: Option<u32>,
}

#[tarpc::service]
pub trait BatmanNeighborsServer {
    async fn get_neighbors() -> Result<Vec<BatmanNeighbor>, String>;
}
