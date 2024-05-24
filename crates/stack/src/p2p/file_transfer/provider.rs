use std::{
    io,
    path::{Path, PathBuf},
    sync::{atomic::AtomicU64, Arc},
};

use asynchronous_codec::{CborCodec, Framed};
use futures::SinkExt;
use libp2p::Stream;
use libp2p_stream::Control;
use tokio::{
    fs::{try_exists, File},
    io::{split, AsyncReadExt},
};
use tokio_stream::StreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::p2p::file_transfer::{Request, Response};

use super::{ack::ack_writer, bar_style, Cid, ExistenceInfo, STREAM_PROTOCOL};

pub struct FileTransferProvider {
    directory: PathBuf,
    control: Control,
    total_streams: AtomicU64,
    streams_per_cid: dashmap::DashMap<Cid, usize>,
}

impl FileTransferProvider {
    pub fn new(directory: PathBuf, control: Control) -> Self {
        Self {
            directory,
            control,
            total_streams: AtomicU64::new(0),
            streams_per_cid: dashmap::DashMap::new(),
        }
    }

    pub async fn run(mut self) {
        let mut streams = self
            .control
            .accept(STREAM_PROTOCOL)
            .expect("Already registered stream (likely two file transfer providers)");

        let stream_handler = Arc::new(StreamHandler {
            directory: self.directory,
            total_streams: self.total_streams,
            streams_per_cid: self.streams_per_cid,
        });

        while let Some((_peer_id, stream)) = streams.next().await {
            let local_stream_handler = Arc::clone(&stream_handler);
            tokio::spawn(async move {
                let mut cid = None;
                if let Err(e) = local_stream_handler
                    .handle_stream_inner(stream, &mut cid)
                    .await
                {
                    tracing::trace!(error = ?e, "Error handling stream");
                }
                if let Some(cid) = cid {
                    local_stream_handler
                        .total_streams
                        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    local_stream_handler
                        .streams_per_cid
                        .entry(cid)
                        .and_modify(|e| *e -= 1)
                        .or_insert(0);
                }
            });
        }
    }
}

async fn get_file(directory: &Path, cid: Cid) -> io::Result<Option<File>> {
    let path = directory.join::<PathBuf>(cid.into());
    if try_exists(&path).await? {
        Ok(Some(File::open(path).await?))
    } else {
        Ok(None)
    }
}

struct StreamHandler {
    directory: PathBuf,
    total_streams: AtomicU64,
    streams_per_cid: dashmap::DashMap<Cid, usize>,
}

impl StreamHandler {
    async fn handle_stream_inner(
        &self,
        stream: Stream,
        streaming_cid: &mut Option<Cid>,
    ) -> anyhow::Result<()> {
        let mut framed = Framed::new(stream, CborCodec::<Response, Request>::new());
        let cid = match framed.next().await {
            Some(Ok(Request::GetCid(cid))) => cid,
            Some(Err(e)) => Err(e)?,
            None => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "No request"))?,
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid request",
            ))?,
        };

        let Some(file) = get_file(&self.directory, cid).await? else {
            framed.send(Response::Cid(None)).await?;
            return Ok(());
        };

        let length = file.metadata().await?.len();
        let streams_on_cid = self.streams_per_cid.get(&cid).map_or(0, |e| *e);
        let total_streams = self
            .total_streams
            .load(std::sync::atomic::Ordering::Relaxed);

        framed
            .send(Response::Cid(Some(ExistenceInfo {
                total_streams,
                streams_on_cid: streams_on_cid as u64,
                length,
            })))
            .await?;

        let mut framed_parts = match framed.next().await {
            Some(Ok(Request::StartStream)) => framed.into_parts(),
            Some(Err(e)) => Err(e)?,
            None => return Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid request",
            ))?,
        };

        // let mut hashing_read = HashingRead::new(file);
        *streaming_cid = Some(cid);

        self.total_streams
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.streams_per_cid
            .entry(cid)
            .and_modify(|e| *e += 1)
            .or_insert(1);

        let file = file.take(length);
        let file = framed_parts.write_buffer.as_mut().chain(file);

        let multi = indicatif::MultiProgress::new();
        let file = multi
            .add(indicatif::ProgressBar::new(length))
            .with_prefix("File")
            .with_style(bar_style())
            .wrap_async_read(file);

        let (mut reader, writer) = split((&mut framed_parts.io).compat());
        let mut writer = multi
            .add(indicatif::ProgressBar::new(length))
            .with_prefix("Network")
            .with_style(bar_style())
            .wrap_async_write(writer);
        ack_writer(&mut reader, &mut writer, file).await?;

        let _ = multi.clear();

        let mut framed = Framed::from_parts(framed_parts);

        match framed.next().await {
            Some(Ok(Request::Ok)) | None => Ok(()),
            Some(Err(e)) => Err(e)?,
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid response",
            ))?,
        }
    }
}
