#[cfg(feature = "network")]
use std::ffi::OsString;
use std::path::PathBuf;

#[cfg(feature = "network")]
use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    response::IntoResponse,
    BoxError, Json,
};
use const_format::concatcp;
#[cfg(feature = "network")]
use futures::{Stream, TryStreamExt as _};
use hyveos_core::grpc::{self, file_transfer_server::FileTransfer};
#[cfg(feature = "network")]
use hyveos_core::{file_transfer::Cid, serde::JsonResult};
#[cfg(feature = "network")]
use hyveos_p2p_stack::file_transfer::ClientError;
use hyveos_p2p_stack::Client;
#[cfg(feature = "network")]
use tokio::{fs::File, io::BufWriter};
#[cfg(feature = "network")]
use tokio_util::io::{ReaderStream, StreamReader};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{TonicResult, CONTAINER_SHARED_DIR};

pub struct FileTransferServer {
    client: Client,
    shared_dir_path: PathBuf,
}

impl FileTransferServer {
    pub fn new(client: Client, shared_dir_path: PathBuf) -> Self {
        Self {
            client,
            shared_dir_path,
        }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl FileTransfer for FileTransferServer {
    async fn publish_file(&self, request: TonicRequest<grpc::FilePath>) -> TonicResult<grpc::Cid> {
        let file_path = request.into_inner();

        tracing::debug!(request=?file_path, "Received publish_file request");

        let container_file_path = PathBuf::from(file_path);

        let file_path = self.shared_dir_path.join(
            container_file_path
                .strip_prefix(CONTAINER_SHARED_DIR)
                .map_err(|_| {
                    Status::invalid_argument(concatcp!(
                        "File must be in shared directory (",
                        CONTAINER_SHARED_DIR,
                        ")"
                    ))
                })?,
        );

        self.client
            .file_transfer()
            .import_new_file(&file_path)
            .await
            .map(Into::into)
            .map(TonicResponse::new)
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_file(&self, request: TonicRequest<grpc::Cid>) -> TonicResult<grpc::FilePath> {
        let cid = request.into_inner();

        tracing::debug!(request=?cid, "Received get_file request");

        let ulid_string = cid.id.ulid.clone();

        let store_path = self
            .client
            .file_transfer()
            .get_cid(cid.try_into()?)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        tokio::fs::copy(store_path, self.shared_dir_path.join(&ulid_string))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let container_file_path = PathBuf::from(CONTAINER_SHARED_DIR).join(ulid_string);

        Ok(TonicResponse::new(container_file_path.try_into()?))
    }
}

#[cfg(feature = "network")]
#[derive(Clone)]
pub struct FileTransferHTTPServer {
    client: Client,
    shared_dir_path: PathBuf,
}

#[cfg(feature = "network")]
impl FileTransferHTTPServer {
    pub fn new(client: Client, shared_dir_path: PathBuf) -> Self {
        Self {
            client,
            shared_dir_path,
        }
    }

    pub async fn publish_file(
        State(this): State<Self>,
        Path(file_name): Path<String>,
        request: Request,
    ) -> Json<JsonResult<Cid, ClientError>> {
        let result = this
            .publish_file_impl(file_name, request.into_body().into_data_stream())
            .await;

        Json(result.into())
    }

    #[tracing::instrument(skip(self, stream))]
    async fn publish_file_impl<E>(
        &self,
        file_name: String,
        stream: impl Stream<Item = Result<Bytes, E>>,
    ) -> Result<Cid, ClientError>
    where
        E: Into<BoxError>,
    {
        let file_name = PathBuf::from(file_name);
        let file_stem = file_name.file_stem().unwrap_or_default().to_os_string();
        let file_ext = file_name.extension().map(OsString::from);

        let mut file_path = self.shared_dir_path.join(file_name);
        for i in 1.. {
            if file_path.exists() {
                break;
            }

            let mut file_name = file_stem.clone();
            file_name.push(format!("_{i}"));
            if let Some(ext) = file_ext.clone() {
                file_name.push(".");
                file_name.push(ext);
            }

            file_path = self.shared_dir_path.join(file_name);
        }

        let reader = StreamReader::new(
            stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)),
        );
        futures::pin_mut!(reader);

        let mut file = BufWriter::new(File::create(&file_path).await?);
        tokio::io::copy(&mut reader, &mut file).await?;

        self.client
            .file_transfer()
            .import_new_file(&file_path)
            .await
            .map(Into::into)
    }

    pub async fn get_file(State(this): State<Self>, Query(cid): Query<Cid>) -> impl IntoResponse {
        this.get_file_impl(cid)
            .await
            .map(|body| ([(header::CONTENT_TYPE, "application/octet-stream")], body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }

    #[tracing::instrument(skip(self, cid))]
    async fn get_file_impl(&self, cid: Cid) -> Result<Body, ClientError> {
        let store_path = self.client.file_transfer().get_cid(cid).await?;

        let bytes = File::open(store_path).await?;
        let stream = ReaderStream::new(bytes);

        Ok(Body::from_stream(stream))
    }
}
