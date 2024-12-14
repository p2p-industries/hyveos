use std::{
    io,
    path::{Path, PathBuf},
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

/// # Errors
///
/// Returns an error if the file does not exist or if the file is not a regular file or the
/// permissions couldn't be set or read.
pub async fn read_only_hard_link(src: &Path, dest: &Path) -> io::Result<()> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    // Set source file read only
    let src_file = tokio::fs::File::open(src).await?;
    let metadata = src_file.metadata().await?;
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    #[cfg(unix)]
    permissions.set_mode(0o444);
    tokio::fs::set_permissions(src, permissions.clone()).await?;
    // Create hardlink to source file
    tokio::fs::hard_link(src, dest).await?;
    // Set permissions to read only
    tokio::fs::set_permissions(dest, permissions).await?;
    Ok(())
}
