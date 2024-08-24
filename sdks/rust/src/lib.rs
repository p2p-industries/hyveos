#![warn(clippy::pedantic, clippy::expect_used, clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! # P2P Industries SDK
//!
//! This crate provides a high-level API for interacting with the P2P Industries stack.
//!
//! > **Note**: This crate will only work properly when running inside a docker container, started
//! > by the P2P Industries stack.
//!
//! ## Crate Features
//!
//! - `json` - Enables JSON (de-)serialization for data sent to other nodes.
//! - `cbor` - Enables CBOR (de-)serialization for data sent to other nodes.
//!
//! ## Example
//!
//! ```no_run
//! use futures::StreamExt as _;
//! use p2p_industries_sdk::P2PConnection;
//!
//! #[tokio::main]
//! async fn main() {
//!     let connection = P2PConnection::get().await.unwrap();
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

pub use libp2p_identity::PeerId;

#[doc(inline)]
pub use crate::{connection::P2PConnection, error::Error};

pub mod connection;
pub mod error;
pub mod services;
