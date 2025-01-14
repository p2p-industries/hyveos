use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
    sync::Arc,
};
#[cfg(feature = "network")]
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt as _, TryStreamExt as _};
pub use hyveos_core::file_transfer::DownloadEvent;
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
use tokio::{
    io::{BufWriter, ReadBuf},
    sync::mpsc,
};
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
/// let cid = file_transfer_service.publish(&file_path).await.unwrap();
///
/// println!("Content ID: {cid:?}");
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Service {
    client: Client,
    shared_dir_path: Option<Arc<PathBuf>>,
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

        Self {
            client,
            shared_dir_path: connection.shared_dir_path.clone(),
        }
    }

    /// Publishes a file in the mesh network and returns its content ID.
    ///
    /// Before it's published, the file is copied to the shared directory if it is not already
    /// there. By default, the shared directory is defined by the `HYVEOS_BRIDGE_SHARED_DIR`
    /// environment variable ([`hyveos_core::BRIDGE_SHARED_DIR_ENV_VAR`]). However, it can be
    /// set to a custom path when using a custom connection ([`ConnectionBuilder::custom`]).
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
    /// let cid = file_transfer_service.publish(&file_path).await.unwrap();
    ///
    /// println!("Content ID: {cid:?}");
    /// # }
    /// ```
    ///
    /// [`ConnectionBuilder::custom`]: crate::connection::ConnectionBuilder::custom
    #[tracing::instrument(skip(self, path), fields(path = %path.as_ref().display()))]
    pub async fn publish(&mut self, path: impl AsRef<Path>) -> Result<Cid> {
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
                    "file-transfer/publish/{}",
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

        let shared_dir = if let Some(shared_dir) = &self.shared_dir_path {
            Cow::Borrowed(shared_dir.as_ref())
        } else {
            Cow::Owned(
                env::var(BRIDGE_SHARED_DIR_ENV_VAR)
                    .map_err(|e| Error::EnvVarMissing(BRIDGE_SHARED_DIR_ENV_VAR, e))?
                    .into(),
            )
        };

        let path: FilePath = if path.starts_with(shared_dir.as_path()) {
            path.try_into()
        } else {
            let (shared_path, _) = shared_dir.join(file_name).unique_file().await?;

            tokio::fs::copy(&path, &shared_path).await?;

            shared_path.try_into()
        }?;

        client
            .publish(path)
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
    /// use hyveos_sdk::Connection;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// // Let's assume that we can get a content ID of a published file
    /// // from the key-value store topic "file" with the key "example".
    /// let mut kv_service = connection.kv();
    /// let cid = kv_service.get_record_json("file", "example").await.unwrap();
    ///
    /// if let Some(cid) = cid {
    ///     let mut file_transfer_service = connection.file_transfer();
    ///     let path = file_transfer_service.get(cid).await.unwrap();
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
    pub async fn get(&mut self, cid: Cid) -> Result<PathBuf> {
        #[cfg(not(feature = "network"))]
        let client = &mut self.client;
        #[cfg(feature = "network")]
        let client = match &mut self.client {
            Client::Local(client) => client,
            Client::Network(client, url) => {
                let url = url.join("file-transfer/get")?;

                let stream = client.get(url).query(&cid).send().await?.bytes_stream();

                let reader = StreamReader::new(
                    stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err)),
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
            .get(grpc::Cid::from(cid))
            .await
            .map(|response| response.into_inner().into())
            .map_err(Into::into)
    }

    /// Retrieves a file from the mesh network and returns a stream of download events.
    ///
    /// When the local runtime doesn't own a copy of this file yet, it downloads it from one of its peers.
    /// While downloading, it emits events with the download progress as a percentage.
    /// Afterwards, or if it was already locally available, the file is copied
    /// into the shared directory, which is defined by the `HYVEOS_BRIDGE_SHARED_DIR` environment
    /// variable, and the path is emitted as the last event in the stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::StreamExt as _;
    /// use hyveos_sdk::{services::file_transfer::DownloadEvent, Connection};
    /// use indicatif::ProgressBar;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let connection = Connection::new().await.unwrap();
    /// // Let's assume that we can get a content ID of a published file
    /// // from the key-value store topic "file" with the key "example".
    /// let mut kv_service = connection.kv();
    /// let cid = kv_service.get_record_json("file", "example").await.unwrap();
    ///
    /// if let Some(cid) = cid {
    ///     let mut file_transfer_service = connection.file_transfer();
    ///     let mut stream = file_transfer_service
    ///         .get_with_progress(cid)
    ///         .await
    ///         .unwrap();
    ///
    ///     let progress_bar = ProgressBar::new(100);
    ///
    ///     let path = loop {
    ///         match stream.next().await.unwrap().unwrap() {
    ///             DownloadEvent::Progress(progress) => {
    ///                 progress_bar.set_position(progress);
    ///             }
    ///             DownloadEvent::Ready(path) => {
    ///                 progress_bar.finish();
    ///                 break path;
    ///             }
    ///         }
    ///     };
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
    pub async fn get_with_progress(
        &mut self,
        cid: Cid,
    ) -> Result<impl Stream<Item = Result<DownloadEvent>>> {
        #[cfg(not(feature = "network"))]
        let client = &mut self.client;
        #[cfg(feature = "network")]
        let client = match &mut self.client {
            Client::Local(client) => client,
            Client::Network(client, url) => {
                let url = url.join("file-transfer/get")?;

                let (sender, receiver) = mpsc::unbounded_channel();

                tokio::spawn({
                    let client = client.clone();
                    async move {
                        let res = async {
                            let response = client.get(url).query(&cid).send().await?;
                            let length = response.content_length();

                            let stream = response.bytes_stream();

                            let mut reader: Pin<Box<dyn tokio::io::AsyncRead + Send>> =
                                Box::pin(StreamReader::new(
                                    stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err)),
                                ));

                            if let Some(length) = length {
                                reader =
                                    Box::pin(ReadWithProgress::new(reader, length, |progress| {
                                        let _ = sender.send(Ok(DownloadEvent::Progress(progress)));
                                    }));
                            }

                            let (path, file) = env::temp_dir()
                                .join(cid.id.to_string())
                                .unique_file()
                                .await?;

                            let mut file = BufWriter::new(file);

                            tokio::io::copy(&mut reader, &mut file).await?;

                            Ok(DownloadEvent::Ready(path))
                        }
                        .await;

                        let _ = sender.send(res);
                    }
                });

                let stream =
                    tokio_stream::wrappers::UnboundedReceiverStream::new(receiver).right_stream();

                return Ok(stream);
            }
        };

        client
            .get_with_progress(grpc::Cid::from(cid))
            .await
            .map(|response| {
                let stream = response
                    .into_inner()
                    .map_ok(TryInto::try_into)
                    .map(|res| res?.map_err(Into::into));

                #[cfg(feature = "network")]
                let stream = stream.left_stream();

                stream
            })
            .map_err(Into::into)
    }
}

#[cfg(feature = "network")]
#[pin_project::pin_project]
struct ReadWithProgress<R, F> {
    #[pin]
    inner: R,
    total_length: u64,
    progress: F,
}

#[cfg(feature = "network")]
impl<R, F> ReadWithProgress<R, F> {
    fn new(inner: R, total_length: u64, progress: F) -> Self {
        Self {
            inner,
            total_length,
            progress,
        }
    }
}

#[cfg(feature = "network")]
impl<R, F> tokio::io::AsyncRead for ReadWithProgress<R, F>
where
    R: tokio::io::AsyncRead + Unpin,
    F: FnMut(u64),
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        match this.inner.poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                (this.progress)(buf.filled().len() as u64 * 100 / *this.total_length);
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}
