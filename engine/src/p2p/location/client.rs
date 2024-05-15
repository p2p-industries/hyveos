use crate::p2p::client::SpecialClient;

use super::Command;

pub struct Client {
    #[allow(dead_code)]
    inner: SpecialClient<Command>,
}

impl From<SpecialClient<Command>> for Client {
    fn from(inner: SpecialClient<Command>) -> Self {
        Self { inner }
    }
}
