#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! # HyveOS SDK
//!
//! This crate provides a high-level API for interacting with the HyveOS runtime.
//!
//! > **Note**: By default, the [`Connection`] struct assumes that it's running inside a script docker container,
//! > started by the HyveOS runtime.
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
//!     let mut dht_service = connection.dht();
//!     let peer_id = dht_service
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
//!     let data = Result::from(response).unwrap();
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
