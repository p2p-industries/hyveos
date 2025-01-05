use crate::{connection::Connection, error::Result};
use hyveos_core::grpc::{bridge_info_client::BridgeInfoClient, Empty, Version as InnerVersion};
use semver::{Prerelease, Version};
use tonic::transport::Channel;

pub struct Service {
    client: BridgeInfoClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = BridgeInfoClient::new(connection.channel.clone());

        Self { client }
    }

    /// Get the current version of the bridge.
    #[tracing::instrument(skip(self))]
    pub async fn get_version(&mut self) -> Result<Version> {
        let InnerVersion {
            major,
            minor,
            patch,
            pre,
        } = self.client.get_version(Empty {}).await?.into_inner();
        let mut ret = Version::new(major, minor, patch);
        if let Some(pre) = pre {
            if let Ok(pre) = Prerelease::new(&pre) {
                ret.pre = pre;
            }
        }
        Ok(ret)
    }
}
