use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cid {
    pub id: Ulid,
    pub hash: [u8; 32],
}

impl From<Cid> for grpc::Cid {
    fn from(cid: Cid) -> Self {
        Self {
            hash: cid.hash.into(),
            id: cid.id.into(),
        }
    }
}

impl TryFrom<grpc::Cid> for Cid {
    type Error = Error;

    fn try_from(cid: grpc::Cid) -> Result<Self> {
        Ok(Self {
            hash: cid.hash.try_into().map_err(|_| Error::InvalidFileHash)?,
            id: cid.id.try_into()?,
        })
    }
}

impl TryFrom<&Path> for grpc::FilePath {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self> {
        Ok(Self {
            path: path
                .to_str()
                .map(ToString::to_string)
                .ok_or(Error::InvalidFilePath)?,
        })
    }
}

impl TryFrom<PathBuf> for grpc::FilePath {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        Ok(Self {
            path: path
                .to_str()
                .map(ToString::to_string)
                .ok_or(Error::InvalidFilePath)?,
        })
    }
}

impl From<grpc::FilePath> for PathBuf {
    fn from(path: grpc::FilePath) -> Self {
        path.path.into()
    }
}
