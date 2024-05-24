use std::path::{Path, PathBuf};

use tonic::{Request as TonicRequest, Response as TonicResponse, Status};
use ulid::Ulid;

use super::{
    script::{self, file_transfer_server::FileTransfer},
    TonicResult,
};
use crate::p2p::{file_transfer::Cid, Client};

const CONTANIER_SHARED_DIR: &str = "/shared/data";

impl<T: AsRef<Path>> From<T> for script::FilePath {
    fn from(path: T) -> Self {
        Self {
            path: path
                .as_ref()
                .to_str()
                .expect("We can only handle unicode paths at the moment")
                .to_string(),
        }
    }
}

impl From<script::FilePath> for PathBuf {
    fn from(path: script::FilePath) -> Self {
        path.path.into()
    }
}

impl From<Ulid> for script::Id {
    fn from(id: Ulid) -> Self {
        Self {
            ulid: id.to_string(),
        }
    }
}

impl TryFrom<script::Id> for Ulid {
    type Error = Status;

    fn try_from(id: script::Id) -> Result<Self, Status> {
        id.ulid
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Invalid ULID: {e}")))
    }
}

impl From<Cid> for script::Cid {
    fn from(cid: Cid) -> Self {
        Self {
            hash: cid.hash.into(),
            id: cid.id.into(),
        }
    }
}

impl TryFrom<script::Cid> for Cid {
    type Error = Status;

    fn try_from(cid: script::Cid) -> Result<Self, Status> {
        Ok(Self {
            hash: cid
                .hash
                .try_into()
                .map_err(|_| Status::invalid_argument("Invalid file hash: Should be 32 bytes"))?,
            id: cid.id.try_into()?,
        })
    }
}

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

#[tonic::async_trait]
impl FileTransfer for FileTransferServer {
    async fn publish_file(
        &self,
        request: TonicRequest<script::FilePath>,
    ) -> TonicResult<script::Cid> {
        let container_file_path = PathBuf::from(request.into_inner());

        let file_path = self.shared_dir_path.join(
            container_file_path
                .strip_prefix(CONTANIER_SHARED_DIR)
                .map_err(|_| Status::invalid_argument("File must be in shared directory"))?,
        );

        self.client
            .file_transfer()
            .import_new_file(&file_path)
            .await
            .map(Into::into)
            .map(TonicResponse::new)
            .map_err(|e| Status::internal(e.to_string()))
    }

    async fn get_file(&self, request: TonicRequest<script::Cid>) -> TonicResult<script::FilePath> {
        let cid = request.into_inner();

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

        let container_file_path = PathBuf::from(CONTANIER_SHARED_DIR).join(ulid_string);

        Ok(TonicResponse::new(container_file_path.into()))
    }
}
