use std::path::PathBuf;

use const_format::concatcp;
use hyveos_core::grpc::{self, file_transfer_server::FileTransfer};
use hyveos_p2p_stack::Client;
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

        tokio::fs::hard_link(store_path, self.shared_dir_path.join(&ulid_string))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let container_file_path = PathBuf::from(CONTAINER_SHARED_DIR).join(ulid_string);

        Ok(TonicResponse::new(container_file_path.try_into()?))
    }
}
