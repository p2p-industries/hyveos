use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(Arc<libp2p_identity::ParseError>),
    #[error("Invalid ULID: {0}")]
    InvalidUlid(#[from] ulid::DecodeError),
    #[error("Qeury is missing")]
    MissingQuery,
    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("Response is missing")]
    MissingResponse,
    #[error("Event is missing")]
    MissingEvent,
    #[error("Invalid topic: Cannot contain '/'")]
    InvalidTopic,
    #[error("Invalid file hash: Should be 32 bytes")]
    InvalidFileHash,
    #[error("Invalid file path: Contains non-UTF-8 character")]
    InvalidFilePath,
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Invalid cid format")]
    InvalidCidFormat,
}

impl From<libp2p_identity::ParseError> for Error {
    fn from(err: libp2p_identity::ParseError) -> Self {
        Self::InvalidPeerId(Arc::new(err))
    }
}

impl From<Error> for tonic::Status {
    fn from(err: Error) -> Self {
        tonic::Status::invalid_argument(err.to_string())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
