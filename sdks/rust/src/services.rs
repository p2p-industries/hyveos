#[cfg(feature = "cbor")]
#[doc(inline)]
pub use self::req_resp::CborService as CborReqRespService;
#[cfg(feature = "json")]
#[doc(inline)]
pub use self::req_resp::JsonService as JsonReqRespService;
#[doc(hidden)]
#[cfg(feature = "scripting")]
pub use self::scripting::{Config as ScriptingConfig, Service as ScriptingService};
#[doc(inline)]
pub use self::{
    debug::Service as DebugService, dht::Service as DhtService,
    discovery::Service as DiscoveryService, file_transfer::Service as FileTransferService,
    gossipsub::Service as GossipSubService, req_resp::Service as ReqRespService,
};

pub mod debug;
pub mod dht;
pub mod discovery;
pub mod file_transfer;
pub mod gossipsub;
pub mod req_resp;
#[doc(hidden)]
#[cfg(feature = "scripting")]
pub mod scripting;
