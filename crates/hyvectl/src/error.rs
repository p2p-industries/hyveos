use std::string::FromUtf8Error;
use thiserror::Error;
use ulid::DecodeError;
use libp2p_identity::ParseError;
use miette::{Diagnostic};
use hyveos_core::req_resp::ResponseError;

/// The error type for hyvectl with miette diagnostic reporting
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Error, Diagnostic)]
pub enum HyveCtlError {
    /// I/O error
    #[error("I/O error")]
    #[diagnostic(code(hyvectl::io))]
    Io(#[from] std::io::Error),

    /// Int parse error
    #[error("Could not parse integer")]
    #[diagnostic(code(hyvectl::parse_int))]
    ParseInt(#[from] std::num::ParseIntError),

    /// Data parse error
    #[error("Could not parse data")]
    #[diagnostic(code(hyvectl::parse_data))]
    ParseData(#[from] FromUtf8Error),

    /// Peer-Id parse error
    #[error("Could not parse PeerId")]
    #[diagnostic(code(hyvectl::parse_peer_id))]
    ParsePeerId(#[from] ParseError),

    /// Ulid Decode error
    #[error("Could not decode Ulid")]
    #[diagnostic(code(hyvectl::decode_ulid))]
    DecodeUlid(#[from] DecodeError),

    /// HyveOS-SDK error
    #[error("Error from hyveOS")]
    #[diagnostic(code(hyvectl::hyveos_error))]
    Sdk(#[from] hyveos_sdk::Error),

    /// HyveOS-core error
    #[error("Error from hyveOS")]
    #[diagnostic(code(hyvectl::hyveos_error))]
    Core(#[from] hyveos_core::Error),

    /// Response error
    #[error("Response error")]
    #[diagnostic(code(hyvectl::response_error))]
    Response(#[from] ResponseError),
}

pub type HyveCtlResult<T> = Result<T, HyveCtlError>;

