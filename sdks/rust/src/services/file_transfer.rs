use std::{
    env,
    path::{Path, PathBuf},
};

use hyveos_core::{
    file_transfer::Cid,
    grpc::{self, file_transfer_client::FileTransferClient, FilePath},
};
use tokio::fs::File;
use tonic::transport::Channel;

use crate::{
    connection::Connection,
    error::{Error, Result},
};

trait PathExt {
    async fn unique_file(&self) -> Result<(PathBuf, File)>;
}

impl PathExt for Path {
    async fn unique_file(&self) -> Result<(PathBuf, File)> {
        let file_stem = self
            .file_stem()
            .ok_or_else(|| Error::NoFileName(self.to_owned()))?
            .to_owned();

        let extension = self.extension();

        let mut i = 1;
        let mut new_path = self.to_owned();

        loop {
            if let Ok(file) = File::create_new(&new_path).await {
                return Ok((new_path, file));
            }

            let mut file_name = file_stem.clone();
            file_name.push(format!("-{i}"));

            if let Some(extension) = extension {
                file_name.push(".");
                file_name.push(extension);
            }

            new_path.set_file_name(file_name);

            i += 1;
        }
    }
}

/// A handle to the file transfer service.
///
/// Exposes methods to interact with the file transfer service, like for publishing and getting
/// files.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// use hyveos_sdk::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let shared_dir = std::env::var("P2P_INDUSTRIES_SHARED_DIR").unwrap();
/// let file_path = Path::new(&shared_dir).join("example.txt");
/// tokio::fs::write(&file_path, "Hello, world!").await.unwrap();
///
/// let connection = Connection::new().await.unwrap();
/// let mut file_transfer_service = connection.file_transfer();
/// let cid = file_transfer_service.publish_file(&file_path).await.unwrap();
///
/// println!("Content ID: {cid:?}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: FileTransferClient<Channel>,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        let client = FileTransferClient::new(connection.channel.clone());

        Self { client }
    }

    /// Publishes a file in the mesh network and returns its content ID.
    ///
    /// Before it's published, the file is copied to the shared directory if it is not already
    /// there. The shared directory is defined by the `HYVEOS_BRIDGE_SHARED_DIR` environment
    /// variable ([`hyveos_core::BRIDGE_SHARED_DIR_ENV_VAR`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let shared_dir = std::env::var("P2P_INDUSTRIES_SHARED_DIR").unwrap();
    /// let file_path = Path::new(&shared_dir).join("example.txt");
    /// tokio::fs::write(&file_path, "Hello, world!").await.unwrap();
    ///
    /// let connection = Connection::new().await.unwrap();
    /// let mut file_transfer_service = connection.file_transfer();
    /// let cid = file_transfer_service.publish_file(&file_path).await.unwrap();
    ///
    /// println!("Content ID: {cid:?}");
    /// # }
    /// ```
    #[tracing::instrument(skip(self, path), fields(path = %path.as_ref().display()))]
    pub async fn publish_file(&mut self, path: impl AsRef<Path>) -> Result<Cid> {
        let shared_dir = env::var("P2P_INDUSTRIES_SHARED_DIR")
            .map_err(|e| Error::EnvVarMissing("P2P_INDUSTRIES_SHARED_DIR", e))?;

        let path = path.as_ref().canonicalize()?;

        let path: FilePath = if path.starts_with(&shared_dir) {
            path.try_into()
        } else {
            let Some(file_name) = path.file_name() else {
                return Err(Error::NoFileName(path));
            };

            let (shared_path, _) = Path::new(&shared_dir).join(file_name).unique_file().await?;

            tokio::fs::copy(&path, &shared_path).await?;

            shared_path.try_into()
        }?;

        self.client
            .publish_file(path)
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }

    /// Publishes a file in the mesh network and returns its content ID.
    ///
    /// Before it's published, the file is copied to the shared directory if it is not already
    /// there. The shared directory is defined by the `P2P_INDUSTRIES_SHARED_DIR` environment
    /// variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hyveos_core::file_transfer::Cid;
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// let mut dht_service = connection.dht();
    /// let cid = dht_service.get_record_json("file", "example").await.unwrap();
    ///
    /// if let Some(cid) = cid {
    ///     let mut file_transfer_service = connection.file_transfer();
    ///     let path = file_transfer_service.get_file(cid).await.unwrap();
    ///
    ///     println!("File path: {}", path.display());
    ///
    ///     let contents = tokio::fs::read_to_string(path).await.unwrap();
    ///     println!("File length: {}", contents.len());
    /// } else {
    ///     println!("File not found");
    /// }
    /// # }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn get_file(&mut self, cid: Cid) -> Result<PathBuf> {
        tracing::debug!("Received get_file request");

        self.client
            .get_file(grpc::Cid::from(cid))
            .await
            .map(|response| response.into_inner().into())
            .map_err(Into::into)
    }
}
