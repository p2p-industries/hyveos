mod behaviour;

use std::{io, sync::Arc, time::Duration};

pub use behaviour::{Behaviour, Event};
use libp2p::{Multiaddr, PeerId};
use macaddress::MacAddress;

fn if_name_to_index(name: impl Into<Vec<u8>>) -> io::Result<u32> {
    let ifname = std::ffi::CString::new(name)?;
    match unsafe { libc::if_nametoindex(ifname.as_ptr()) } {
        0 => Err(io::Error::last_os_error()),
        otherwise => Ok(otherwise),
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub batman_if_index: u32,
    pub socket_path: Arc<str>,
    pub refresh_interval: Duration,
    pub neighbour_timeout: Duration,
    pub request_retries: u32,
    pub request_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            batman_if_index: if_name_to_index("bat0").expect("Failed to resolve bat0"),
            socket_path: "/var/run/batman-neighbours.sock".into(),
            refresh_interval: Duration::from_secs(1),
            neighbour_timeout: Duration::from_secs(10),
            request_retries: 3,
            request_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedNeighbour {
    pub peer_id: PeerId,
    pub if_index: u32,
    pub mac: MacAddress,
    pub batman_addr: Multiaddr,
    pub direct_addr: Multiaddr,
}
