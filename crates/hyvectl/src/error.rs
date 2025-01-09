use std::string::FromUtf8Error;
use thiserror::Error;
use ulid::DecodeError;
use libp2p_identity::ParseError;
use hyveos_sdk;
use hyveos_core;
use miette::{Diagnostic};
use hyveos_core::req_resp::ResponseError;

/// The error type for hyvectl with miette diagnostic reporting
#[derive(Debug, Error, Diagnostic)]
pub enum HyveCtlError {
    /// I/O error
    #[error("I/O error")]
    #[diagnostic(code(hyvectl::io))]
    IoError(#[from] std::io::Error),

    /// Int parse error
    #[error("Could not parse integer")]
    #[diagnostic(code(hyvectl::parse_int))]
    ParseIntError(#[from] std::num::ParseIntError),

    /// Data parse error
    #[error("Could not parse data")]
    #[diagnostic(code(hyvectl::parse_data))]
    ParseDataError(#[from] FromUtf8Error),

    /// Peer-Id parse error
    #[error("Could not parse PeerId")]
    #[diagnostic(code(hyvectl::parse_peer_id))]
    ParsePeerIdError(#[from] ParseError),

    /// Ulid Decode error
    #[error("Could not decode Ulid")]
    #[diagnostic(code(hyvectl::decode_ulid))]
    DecodeUlidError(#[from] DecodeError),

    /// HyveOS-SDK error
    #[error("Error from hyveOS")]
    #[diagnostic(code(hyvectl::hyveos_error))]
    SKDError(#[from] hyveos_sdk::Error),

    /// HyveOS-core error
    #[error("Error from hyveOS")]
    #[diagnostic(code(hyvectl::hyveos_error))]
    CoreError(#[from] hyveos_core::Error),

    /// Response error
    #[error("Response error")]
    #[diagnostic(code(hyvectl::response_error))]
    ResponseError(#[from] ResponseError),
}

pub type HyveCtlResult<T> = Result<T, HyveCtlError>;

