use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ulid_str = self.id.to_string();
        let hash_hex = hex::encode(self.hash);
        write!(f, "{}-{}", ulid_str, hash_hex)
    }
}

impl FromStr for Cid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.splitn(2, '-');
        let ulid_part = parts.next().ok_or(Error::InvalidCidFormat)?;
        let hash_part = parts.next().ok_or(Error::InvalidCidFormat)?;

        let id = Ulid::from_string(ulid_part).map_err(|_| Error::InvalidCidFormat)?;

        let hash_bytes = hex::decode(hash_part).map_err(|_| Error::InvalidCidFormat)?;
        if hash_bytes.len() != 32 {
            return Err(Error::InvalidCidFormat);
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);

        Ok(Cid { id, hash: hash_array })
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DownloadEvent {
    Progress(u64),
    Ready(PathBuf),
}

impl TryFrom<DownloadEvent> for grpc::DownloadEvent {
    type Error = Error;

    fn try_from(event: DownloadEvent) -> Result<Self> {
        let event = match event {
            DownloadEvent::Progress(progress) => grpc::download_event::Event::Progress(progress),
            DownloadEvent::Ready(path) => grpc::download_event::Event::Ready(path.try_into()?),
        };

        Ok(Self { event: Some(event) })
    }
}

impl TryFrom<grpc::DownloadEvent> for DownloadEvent {
    type Error = Error;

    fn try_from(event: grpc::DownloadEvent) -> Result<Self> {
        Ok(match event.event.ok_or(Error::MissingEvent)? {
            grpc::download_event::Event::Progress(progress) => DownloadEvent::Progress(progress),
            grpc::download_event::Event::Ready(path) => DownloadEvent::Ready(path.into()),
        })
    }
}
