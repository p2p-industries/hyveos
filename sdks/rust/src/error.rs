use std::{io, path::PathBuf};

use tonic::{transport::Error as TransportError, Status};

/// The error type for the SDK.
///
/// This error type is used throughout the SDK to represent all possible errors that can occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// A Tonic transport error occurred.
    #[error("Tonic transport error: {0}")]
    Transport(#[from] TransportError),
    /// A Tonic status error occurred.
    #[error("Tonic error: {0}")]
    Status(#[from] Status),
    /// An error occurred while serializing or deserializing data to/from JSON.
    #[cfg(feature = "json")]
    #[error("JSON (de-)serialization error: {0}")]
    Json(#[from] serde_json::Error),
    /// An error occurred while serializing or deserializing data to/from CBOR.
    #[cfg(feature = "cbor")]
    #[error("CBOR (de-)serialization error: {0}")]
    Cbor(#[from] serde_cbor::Error),
    /// An error occurred while parsing a `PeerId`.
    #[error("PeerId parsing error: {0}")]
    PeerId(#[from] libp2p_identity::ParseError),
    /// An error from the core library.
    #[error(transparent)]
    Core(#[from] hyveos_core::Error),
    /// An environment variable was expected to be set, but it wasn't.
    #[error("Could not get {0}: {1}")]
    EnvVarMissing(&'static str, #[source] std::env::VarError),
    /// A path was expected to have a file name, but it didn't.
    #[error("Path has no file name: {}", .0.display())]
    NoFileName(PathBuf),
    /// An error occurred while parsing a URI.
    #[cfg(feature = "network")]
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    /// An error occurred while parsing URI parts.
    #[cfg(feature = "network")]
    #[error("Invalid URI: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
    /// An error occurred while parsing a URL.
    #[cfg(feature = "network")]
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[cfg(feature = "network")]
    /// An error occurred while sending a request.
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "network")]
    /// Got an error response from a HTTP request.
    #[error("Got an error response: {0}")]
    Response(String),
}

/// Alias for a `Result` that defaults to [`Error`] as the error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
