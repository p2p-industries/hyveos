#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! # hyveOS SDK
//!
//! This crate provides a high-level API for interacting with the hyveOS runtime.
//!
//! > **Note**: By default, the [`Connection`] struct assumes that it's running
//! > inside an application docker container, started by the hyveOS runtime.
//! > To use the SDK elsewhere, the connection can be configured using [`ConnectionBuilder`].
//!
//! ## Crate Features
//!
//! - `json` - Enables JSON (de-)serialization for data exchanged with other nodes.
//! - `cbor` - Enables CBOR (de-)serialization for data exchanged with other nodes.
//!
//! ## Example
//!
//! ```no_run
//! use futures::StreamExt as _;
//! use hyveos_sdk::Connection;
//!
//! #[tokio::main]
//! async fn main() {
//!     let connection = Connection::new().await.unwrap();
//!     let mut discovery_service = connection.discovery();
//!     let peer_id = discovery_service
//!         .get_providers("identification", "example")
//!         .await
//!         .unwrap()
//!         .next()
//!         .await
//!         .unwrap()
//!         .unwrap();
//!
//!     let mut req_resp_service = connection.req_resp();
//!     let response = req_resp_service
//!         .send_request(peer_id, "Hello, world!", None)
//!         .await
//!         .unwrap();
//!
//!     let data = Vec::try_from(response).unwrap();
//!     println!("Received response: {}", String::from_utf8(data).unwrap());
//! }
//! ```

#[cfg(feature = "network")]
pub use http::Uri;
pub use libp2p_identity::PeerId;

#[doc(inline)]
pub use crate::{
    connection::{Connection, ConnectionBuilder},
    error::Error,
};

pub mod connection;
pub mod error;
pub mod services;
