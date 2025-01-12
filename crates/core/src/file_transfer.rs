use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

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
        write!(f, "{ulid_str}-{hash_hex}")
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

        Ok(Cid {
            id,
            hash: hash_array,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_cid_to_string() {
        let cid = Cid {
            id: Ulid::from_string("01GZMC49M8599PQPNGDSAX6X1F").unwrap(),
            hash: [
                0xFF, 0xD2, 0x34, 0x1A, 0x00, 0xAB, 0xCD, 0xEF, 0x00, 0x11, 0x33, 0x55, 0x77, 0x99,
                0xBB, 0xDD, 0xCC, 0xEE, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
                0xFF, 0x12, 0x34, 0x56,
            ],
        };

        let s = cid.to_string();

        let expected_str = "01GZMC49M8599PQPNGDSAX6X1F-ffd2341a00abcdef001133557799bbddccee5566778899aabbccddeeff123456";

        assert_eq!(s, expected_str, "Cid::to_string failed");
    }

    #[test]
    fn test_cid_from_str() {
        let input_str = "01GZMC49M8599PQPNGDSAX6X1F-ffd2341a00abcdef001133557799bbddccee5566778899aabbccddeeff123456";

        let cid = Cid::from_str(input_str).expect("Parsing valid CID");

        assert_eq!(
            cid.id,
            Ulid::from_string("01GZMC49M8599PQPNGDSAX6X1F").unwrap(),
            "The ULID part was incorrectly parsed"
        );

        let expected_hash = [
            0xFF, 0xD2, 0x34, 0x1A, 0x00, 0xAB, 0xCD, 0xEF, 0x00, 0x11, 0x33, 0x55, 0x77, 0x99,
            0xBB, 0xDD, 0xCC, 0xEE, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
            0xFF, 0x12, 0x34, 0x56,
        ];
        assert_eq!(
            cid.hash, expected_hash,
            "The hash part was incorrectly parsed"
        );
    }
}
