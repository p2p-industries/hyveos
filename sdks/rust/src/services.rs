pub use hyveos_core::{debug::MeshTopologyEvent, neighbours::NeighbourEvent};

#[doc(hidden)]
#[cfg(feature = "app-management")]
pub use self::apps::{Config as AppConfig, Service as AppsService};
#[cfg(feature = "cbor")]
#[doc(inline)]
pub use self::req_resp::CborService as CborReqRespService;
#[cfg(feature = "json")]
#[doc(inline)]
pub use self::req_resp::JsonService as JsonReqRespService;
#[doc(inline)]
pub use self::{
    debug::Service as DebugService, discovery::Service as DiscoveryService,
    file_transfer::Service as FileTransferService, kv::Service as DhtService,
    local_kv::Service as DbService, neighbours::Service as NeighboursService,
    pub_sub::Service as GossipSubService, req_resp::Service as ReqRespService,
};

#[doc(hidden)]
#[cfg(feature = "app-management")]
pub mod apps;
pub mod debug;
pub mod discovery;
pub mod file_transfer;
pub mod kv;
pub mod local_kv;
pub mod neighbours;
pub mod pub_sub;
pub mod req_resp;
