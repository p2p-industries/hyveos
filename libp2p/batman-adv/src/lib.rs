mod behaviour;

use std::{sync::Arc, time::Duration};

pub use behaviour::{Behaviour, Event};

#[derive(Debug, Clone)]
pub struct Config {
    pub interface: Arc<str>,
    pub socket_path: Arc<str>,
    pub refresh_interval: Duration,
    pub neighbour_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interface: "bat0".into(),
            socket_path: "/var/run/batman-neighbours.sock".into(),
            refresh_interval: Duration::from_secs(1),
            neighbour_timeout: Duration::from_secs(10),
        }
    }
}
