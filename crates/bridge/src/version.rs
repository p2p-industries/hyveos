use hyveos_core::grpc::{bridge_info_server::BridgeInfo, Empty, Version};

use crate::TonicResult;

pub struct VersionServer;

fn get_version() -> Result<Version, std::num::ParseIntError> {
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse()?;
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse()?;
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse()?;
    let pre = option_env!("CARGO_PKG_VERSION_PRE").map(|e| e.to_string());
    Ok(Version {
        major,
        minor,
        patch,
        pre,
    })
}

#[tonic::async_trait]
impl BridgeInfo for VersionServer {
    async fn get_version(&self, _: tonic::Request<Empty>) -> TonicResult<Version> {
        get_version()
            .map(|e| tonic::Response::new(e))
            .map_err(|e| tonic::Status::internal(format!("Failed to parse version: {e}")))
    }
}
