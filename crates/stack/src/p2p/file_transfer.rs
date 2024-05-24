use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_once_cell::OnceCell;
use asynchronous_codec::{CborCodec, CborCodecError, Framed, FramedParts};
use base64_simd::{Out, URL_SAFE};
use futures::{SinkExt, Stream, StreamExt};
use indicatif::ProgressFinish;
use libp2p::{
    kad::{AddProviderError, GetProvidersOk, RecordKey},
    PeerId, StreamProtocol,
};
use libp2p_stream::{Behaviour, Control, OpenStreamError};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    fs::File,
    io::{split, AsyncReadExt, AsyncWriteExt, ReadBuf},
    sync::{oneshot, Mutex},
};
use tokio_stream::{iter, wrappers::ReadDirStream};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use ulid::Ulid;

use crate::{impl_from_special_command, p2p::file_transfer::ack::ack_reader};

pub use self::provider::FileTransferProvider;

use super::{
    actor::SubActor,
    client::{RequestError, SpecialClient},
    kad,
};

#[cfg(feature = "batman")]
use super::neighbours;

mod ack;
mod provider;

/// The top k providers to query for a file.
const TOP_K: usize = 10;

pub fn new() -> Behaviour {
    Behaviour::new()
}

pub enum Command {
    GetControl {
        sender: oneshot::Sender<Control>,
    },
    SetDirectory {
        directory: PathBuf,
    },
    GetDirectory {
        sender: oneshot::Sender<Option<PathBuf>>,
    },
}

impl_from_special_command!(FileTransfer);

#[derive(Debug, Default)]
pub struct Actor {
    directory: Option<PathBuf>,
}

impl SubActor for Actor {
    type Event = ();
    type SubCommand = Command;
    type EventError = void::Void;
    type CommandError = void::Void;

    fn handle_command(
        &mut self,
        command: Self::SubCommand,
        behaviour: &mut super::behaviour::MyBehaviour,
    ) -> Result<(), Self::CommandError> {
        match command {
            Command::GetControl { sender } => {
                let control = behaviour.file_transfer.new_control();
                let _ = sender.send(control);
            }
            Command::SetDirectory { directory } => {
                self.directory = Some(directory);
            }
            Command::GetDirectory { sender } => {
                let _ = sender.send(self.directory.clone());
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Cid {
    pub id: Ulid,
    pub hash: [u8; 32],
}

const STREAM_PROTOCOL: StreamProtocol = StreamProtocol::new("/file-transfer/0.1.0");

impl From<Cid> for PathBuf {
    fn from(Cid { id, hash }: Cid) -> Self {
        let mut out = [0u8; 64];
        let hash = URL_SAFE.encode_as_str(&hash[..], Out::from_slice(&mut out[..]));
        debug_assert!(!hash.contains('/') && !hash.contains('+'));
        let mut path = PathBuf::from(format!("{id}+{hash}"));
        path = path.with_extension("data");
        path
    }
}

impl TryFrom<&Path> for Cid {
    type Error = io::Error;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let stem = value
            .file_stem()
            .ok_or(io::ErrorKind::InvalidInput)?
            .to_str()
            .ok_or(io::ErrorKind::InvalidInput)?;
        let mut parts = stem.split('+');
        let id: Ulid = parts
            .next()
            .ok_or(io::ErrorKind::InvalidInput)?
            .parse()
            .map_err(|_| io::ErrorKind::InvalidInput)?;
        let hash_str = parts.next().ok_or(io::ErrorKind::InvalidInput)?;
        let mut hash = [0u8; 32];
        let hash_out = URL_SAFE
            .decode(hash_str.as_bytes(), Out::from_slice(&mut hash[..]))
            .map_err(|_| io::ErrorKind::InvalidInput)?;
        if hash_out.len() != 32 {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(Self { id, hash })
    }
}

impl From<Cid> for RecordKey {
    fn from(value: Cid) -> Self {
        let mut key = [0u8; 16 + 32];
        key[..16].copy_from_slice(&value.id.to_bytes());
        key[16..].copy_from_slice(&value.hash);
        Self::new(&key)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct ExistenceInfo {
    total_streams: u64,
    streams_on_cid: u64,
    length: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Request {
    GetCid(Cid),
    StartStream,
    Ok,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Response {
    Cid(Option<ExistenceInfo>),
}

#[derive(Clone)]
pub struct Client {
    inner: SpecialClient<crate::p2p::command::Command>,
    kademlia: kad::Client,
    #[cfg(feature = "batman")]
    neighbours: neighbours::Client,
    directory: Arc<OnceCell<PathBuf>>,
}

impl From<SpecialClient<crate::p2p::command::Command>> for Client {
    fn from(inner: SpecialClient<crate::p2p::command::Command>) -> Self {
        Self {
            kademlia: SpecialClient::new(inner.sender.clone(), inner.peer_id).into(),
            directory: Arc::new(OnceCell::new()),
            #[cfg(feature = "batman")]
            neighbours: SpecialClient::new(inner.sender.clone(), inner.peer_id).into(),
            inner,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Request error: `{0:?}`")]
    Request(RequestError),
    #[error("IO error: `{0}`")]
    Io(#[from] io::Error),
    #[error("Join error: `{0}`")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Open stream error: `{0}`")]
    OpenStream(#[from] OpenStreamError),
    #[error("Codec error: `{0}`")]
    Codec(#[from] CborCodecError),
    #[error("No providers found")]
    NoProviders,
    #[error("Hash mismatch: expected `{expected:?}`, actual `{actual:?}`")]
    HashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    #[error("Directory not set")]
    DirectoryNotSet,
    #[error("Kademlia error: `{0}`")]
    KadProviding(#[from] RequestError<AddProviderError>),
}

impl Client {
    async fn get_control(&self) -> Result<Control, RequestError> {
        let (sender, receiver) = oneshot::channel();
        self.inner
            .send(Command::GetControl { sender }.into())
            .await
            .map_err(RequestError::Send)?;
        let control = receiver.await.map_err(RequestError::Oneshot)?;
        Ok(control)
    }

    async fn set_directory(&self, directory: PathBuf) -> Result<(), RequestError> {
        self.directory
            .get_or_try_init(async {
                self.inner
                    .send(
                        Command::SetDirectory {
                            directory: directory.clone(),
                        }
                        .into(),
                    )
                    .await
                    .map_err(RequestError::Send)?;
                Ok::<_, RequestError>(directory)
            })
            .await?;
        Ok(())
    }

    pub async fn create_provider(
        &self,
        directory: PathBuf,
    ) -> Result<FileTransferProvider, RequestError> {
        let control = self.get_control().await?;
        self.set_directory(directory.clone()).await?;
        let provider = provider::FileTransferProvider::new(directory, control);
        Ok(provider)
    }

    async fn get_directory(&self) -> Result<&Path, ClientError> {
        Ok(self
            .directory
            .get_or_try_init(async move {
                let (sender, receiver) = oneshot::channel();
                self.inner
                    .send(Command::GetDirectory { sender }.into())
                    .await
                    .map_err(RequestError::Send)
                    .map_err(ClientError::Request)?;
                let path = receiver
                    .await
                    .map_err(RequestError::Oneshot)
                    .map_err(ClientError::Request)?
                    .ok_or(ClientError::DirectoryNotSet)?;
                Ok::<_, ClientError>(path)
            })
            .await?
            .as_path())
    }

    async fn provide_cid(&self, cid: Cid) -> Result<(), ClientError> {
        self.kademlia
            .start_providing(cid.into())
            .await
            .map_err(ClientError::KadProviding)?;
        Ok(())
    }

    pub async fn import_file(&self, cid: Cid, file: &Path) -> Result<(), ClientError> {
        let path = self.get_directory().await?.join::<PathBuf>(cid.into());
        if !file.exists() {
            return Err(io::Error::from(io::ErrorKind::NotFound).into());
        }
        tokio::fs::copy(file, path).await?;
        self.provide_cid(cid).await?;
        Ok(())
    }

    pub async fn import_new_file(&self, path: &Path) -> Result<Cid, ClientError> {
        let mut hasher = Sha256::new();
        let mut file = File::open(path).await?;
        let mut buffer = [0u8; 4096];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let hash = hasher.finalize().into();
        let id = Ulid::new();
        let cid = Cid { id, hash };
        self.import_file(cid, path).await?;
        Ok(cid)
    }

    async fn get_local_file(&self, cid: Cid) -> Result<Option<File>, ClientError> {
        let path = self.get_directory().await?.join::<PathBuf>(cid.into());
        if tokio::fs::try_exists(&path).await? {
            Ok(Some(File::open(path).await?))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "batman")]
    async fn get_neighbours(&self) -> Result<impl Iterator<Item = PeerId>, RequestError> {
        Ok(self.neighbours.get_resolved().await?.into_keys())
    }

    #[cfg(not(feature = "batman"))]
    #[cfg_attr(not(feature = "batman"), allow(clippy::unused_async))]
    async fn get_neighbours(&self) -> Result<impl Iterator<Item = PeerId>, RequestError> {
        Ok(std::iter::empty())
    }

    async fn get_all_providers(&self, cid: Cid) -> Result<(Vec<PeerId>, Vec<PeerId>), ClientError> {
        let all_providers = async {
            let mut providers = self
                .kademlia
                .get_providers(cid.into())
                .await
                .map_err(ClientError::Request)?;
            let mut ret = HashSet::new();
            while let Some(providers) = providers.next().await {
                match providers {
                    Ok(GetProvidersOk::FoundProviders { providers, .. }) => {
                        ret.extend(providers);
                    }
                    Err(e) => {
                        tracing::info!(e = ?e, "Error getting providers");
                    }
                    _ => {}
                }
            }
            Ok::<_, ClientError>(ret.into_iter().collect::<Vec<PeerId>>())
        };
        let neighbours = async {
            Ok::<HashSet<PeerId>, ClientError>(
                self.get_neighbours()
                    .await
                    .map_err(ClientError::Request)?
                    .collect(),
            )
        };

        let (all_providers, neighbours) = tokio::try_join!(all_providers, neighbours)?;

        Ok(all_providers
            .into_iter()
            .partition(|peer| neighbours.contains(peer)))
    }

    async fn get_best_provider(
        &self,
        cid: Cid,
        providers: impl Iterator<Item = PeerId>,
        control: Control,
    ) -> Option<BestProvider> {
        let best_provider = Arc::new(Mutex::new(None::<BestProvider>));
        iter(providers.map(|provider| (provider, control.clone(), best_provider.clone())))
            .map(|(provider, mut control, best_provider)| async move {
                let stream = control
                    .open_stream(provider, STREAM_PROTOCOL)
                    .await
                    .map_err(ClientError::OpenStream)?;
                let provider = tokio::spawn(retrieve_cid(cid, stream));
                Ok::<_, ClientError>((provider.await??, best_provider))
            })
            .for_each_concurrent(None, |future| async move {
                if let Ok((Some(provider), best_provider)) = future.await.inspect_err(|e| {
                    tracing::info!(e = ?e, "Error getting CID");
                }) {
                    let mut best_provider = best_provider.lock().await;
                    match best_provider.as_mut() {
                        Some(BestProvider { score, parts, .. }) => {
                            if provider.score > *score {
                                let _ = futures::io::AsyncWriteExt::close(&mut parts.io).await;
                                *best_provider = Some(provider);
                            }
                        }
                        None => {
                            *best_provider = Some(provider);
                        }
                    }
                }
            })
            .await;
        Arc::into_inner(best_provider)
            .expect("Not all best_provider handles where dropped")
            .into_inner()
    }

    async fn get_first_provider(
        &self,
        cid: Cid,
        providers: impl Iterator<Item = PeerId>,
        control: Control,
    ) -> Option<BestProvider> {
        iter(providers.map(|provider| (provider, control.clone())))
            .map(|(provider, mut control)| async move {
                let stream = control
                    .open_stream(provider, STREAM_PROTOCOL)
                    .await
                    .map_err(ClientError::OpenStream)?;
                tokio::spawn(retrieve_cid(cid, stream)).await?
            })
            .filter_map(|future| {
                Box::pin(async move {
                    match future.await {
                        Ok(Some(best_provider)) => Some(best_provider),
                        Ok(None) => None,
                        Err(e) => {
                            tracing::info!(e = ?e, "Error retrieving CID");
                            None
                        }
                    }
                })
            })
            .next()
            .await
    }

    pub async fn get_cid(&self, cid: Cid) -> Result<PathBuf, ClientError> {
        if (self.get_local_file(cid).await?).is_some() {
            return Ok(self.get_directory().await?.join::<PathBuf>(cid.into()));
        }

        let (neighbours, non_neighbours) = self.get_all_providers(cid).await?;

        let control = self.get_control().await.map_err(ClientError::Request)?;
        let (parts, length) = match self
            .get_best_provider(cid, neighbours.into_iter(), control.clone())
            .await
        {
            Some(BestProvider { parts, length, .. }) => (parts, length),
            None => {
                match self
                    .get_best_provider(
                        cid,
                        non_neighbours.iter().take(TOP_K).copied(),
                        control.clone(),
                    )
                    .await
                {
                    Some(BestProvider { parts, length, .. }) => (parts, length),
                    None => {
                        match self
                            .get_first_provider(
                                cid,
                                non_neighbours.into_iter().skip(TOP_K),
                                control.clone(),
                            )
                            .await
                        {
                            Some(BestProvider { parts, length, .. }) => (parts, length),
                            None => return Err(ClientError::NoProviders),
                        }
                    }
                }
            }
        };

        let path = self.get_directory().await?.join::<PathBuf>(cid.into());
        let mut file = File::create(&path).await?;
        file.sync_all().await?;
        let mut framed = Framed::from_parts(parts);
        framed.send(Request::StartStream).await?;
        let mut parts = framed.into_parts();
        let (reader, writer) = split(parts.io.compat());
        let reader = parts.read_buffer.as_mut().chain(reader).take(length);
        let mut hasher = HashingRead::new(reader);

        let multi = indicatif::MultiProgress::new();
        multi.add(indicatif::ProgressBar::new_spinner());
        let mut file_tracked = multi
            .add(indicatif::ProgressBar::new(length))
            .with_prefix("File")
            .with_style(bar_style())
            .with_message("🚚 Loading...")
            .with_finish(ProgressFinish::WithMessage("✅ Done".into()))
            .wrap_async_write(&mut file);
        let mut stream_tracked = multi
            .add(indicatif::ProgressBar::new(length))
            .with_prefix("Network")
            .with_style(bar_style())
            .with_message("🚚 Loading...")
            .with_finish(ProgressFinish::WithMessage("✅ Done".into()))
            .wrap_async_read(&mut hasher);

        ack_reader(&mut stream_tracked, writer, &mut file_tracked).await?;

        let _ = multi.clear();

        file.flush().await?;
        let hash = hasher.finalize();
        if hash != cid.hash {
            tracing::warn!(actual = ?hash, correct = ?cid.hash, "Hash mismatch");
            tokio::fs::remove_file(&path).await?;
            return Err(ClientError::HashMismatch {
                expected: cid.hash,
                actual: hash,
            });
        }
        file.sync_all().await?;
        file.shutdown().await?;

        self.provide_cid(cid).await?;

        Ok(path)
    }

    pub async fn list(&self) -> Result<impl Stream<Item = io::Result<Cid>>, ClientError> {
        let m = tokio::fs::read_dir(self.get_directory().await?).await?;
        Ok(ReadDirStream::new(m).filter_map(|entry| async move {
            match entry {
                Ok(entry) => match entry.file_type().await {
                    Err(e) => Some(Err(e)),
                    Ok(file_type) if !file_type.is_file() => None,
                    Ok(_) => {
                        let path = entry.path();
                        Some(Cid::try_from(path.as_path()))
                    }
                },
                Err(e) => Some(Err(e)),
            }
        }))
    }
}

struct BestProvider {
    score: u64,
    parts: FramedParts<libp2p::swarm::Stream, CborCodec<Request, Response>>,
    length: u64,
}

async fn retrieve_cid(
    cid: Cid,
    stream: libp2p::swarm::Stream,
) -> Result<Option<BestProvider>, ClientError> {
    let mut framed = Framed::new(stream, CborCodec::<Request, Response>::new());
    framed.send(Request::GetCid(cid)).await?;

    let ExistenceInfo {
        total_streams,
        streams_on_cid,
        length,
    } = match framed.next().await {
        Some(Ok(Response::Cid(Some(info)))) => info,
        Some(Err(e)) => return Err(e.into()),
        _ => return Ok::<_, ClientError>(None),
    };

    let score = total_streams + streams_on_cid;

    Ok(Some(BestProvider {
        score,
        parts: framed.into_parts(),
        length,
    }))
}

#[pin_project]
struct HashingRead<R> {
    #[pin]
    inner: R,
    hasher: sha2::Sha256,
}

impl<R> HashingRead<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
        }
    }

    fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
}

impl<R> tokio::io::AsyncRead for HashingRead<R>
where
    R: tokio::io::AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        let filled_len = buf.filled().len();
        match this.inner.poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                this.hasher.update(&buf.filled()[filled_len..]);
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

fn bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template("{spinner} {message} {prefix:<10}: {wide_bar:.cyan/blue} {decimal_bytes:>10}/{decimal_total_bytes:<10} ({decimal_bytes_per_sec:^10.red}) {elapsed} → {eta}")
        .expect("Invalid template")
        .progress_chars("▰▲▱")
        .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
}
