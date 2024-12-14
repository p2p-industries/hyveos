use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(feature = "network")]
use futures::TryStreamExt as _;
#[cfg(feature = "network")]
use hyveos_core::serde::JsonResult;
use hyveos_core::{
    file_transfer::Cid,
    grpc::{self, file_transfer_client::FileTransferClient, FilePath},
    BRIDGE_SHARED_DIR_ENV_VAR,
};
#[cfg(feature = "network")]
use reqwest::Body;
use tokio::fs::File;
#[cfg(feature = "network")]
use tokio::io::BufWriter;
#[cfg(feature = "network")]
use tokio_util::io::{ReaderStream, StreamReader};
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

#[cfg(not(feature = "network"))]
type Client = FileTransferClient<Channel>;
#[cfg(feature = "network")]
#[derive(Debug, Clone)]
enum Client {
    Local(FileTransferClient<Channel>),
    Network(reqwest::Client, reqwest::Url),
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
/// let shared_dir = std::env::var(hyveos_core::BRIDGE_SHARED_DIR_ENV_VAR).unwrap();
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
    client: Client,
}

impl Service {
    pub(crate) fn new(connection: &Connection) -> Self {
        #[cfg(not(feature = "network"))]
        let client = FileTransferClient::new(connection.channel.clone());
        #[cfg(feature = "network")]
        let client = if let Some((client, url)) = connection.reqwest_client_and_url.clone() {
            Client::Network(client, url)
        } else {
            Client::Local(FileTransferClient::new(connection.channel.clone()))
        };

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
    /// let shared_dir = std::env::var(hyveos_core::BRIDGE_SHARED_DIR_ENV_VAR).unwrap();
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
        let path = path.as_ref().canonicalize()?;

        let Some(file_name) = path.file_name() else {
            return Err(Error::NoFileName(path));
        };

        #[cfg(not(feature = "network"))]
        let client = &mut self.client;
        #[cfg(feature = "network")]
        let client = match &mut self.client {
            Client::Local(client) => client,
            Client::Network(client, url) => {
                let url = url.join(&format!(
                    "file-transfer/publish-file/{}",
                    file_name.to_string_lossy()
                ))?;

                let bytes = File::open(path).await?;
                let stream = ReaderStream::new(bytes);

                let result: Result<Cid, String> = client
                    .post(url)
                    .body(Body::wrap_stream(stream))
                    .send()
                    .await?
                    .json::<JsonResult<_, _>>()
                    .await?
                    .into();

                return result.map_err(Error::Response);
            }
        };

        let shared_dir = env::var(BRIDGE_SHARED_DIR_ENV_VAR)
            .map_err(|e| Error::EnvVarMissing(BRIDGE_SHARED_DIR_ENV_VAR, e))?;

        let path: FilePath = if path.starts_with(&shared_dir) {
            path.try_into()
        } else {
            let (shared_path, _) = Path::new(&shared_dir).join(file_name).unique_file().await?;

            tokio::fs::copy(&path, &shared_path).await?;

            shared_path.try_into()
        }?;

        client
            .publish_file(path)
            .await?
            .into_inner()
            .try_into()
            .map_err(Into::into)
    }

    /// Retrieves a file from the mesh network and returns its path.
    ///
    /// When the local runtime doesn't own a copy of this file yet, it downloads it from one of its peers.
    /// Afterwards, or if it was already locally available, the file is copied
    /// into the shared directory, which is defined by the `HYVEOS_BRIDGE_SHARED_DIR` environment
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
        #[cfg(not(feature = "network"))]
        let client = &mut self.client;
        #[cfg(feature = "network")]
        let client = match &mut self.client {
            Client::Local(client) => client,
            Client::Network(client, url) => {
                let url = url.join("file-transfer/get-file")?;

                let stream = client.get(url).query(&cid).send().await?.bytes_stream();

                let reader = StreamReader::new(
                    stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)),
                );
                futures::pin_mut!(reader);

                let (path, file) = env::temp_dir()
                    .join(cid.id.to_string())
                    .unique_file()
                    .await?;

                let mut file = BufWriter::new(file);

                tokio::io::copy(&mut reader, &mut file).await?;

                return Ok(path);
            }
        };

        client
            .get_file(grpc::Cid::from(cid))
            .await
            .map(|response| response.into_inner().into())
            .map_err(Into::into)
    }
}
