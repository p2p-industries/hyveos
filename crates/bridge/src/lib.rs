#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::{io, path::PathBuf, pin::Pin};

use futures::stream::Stream;
use p2p_industries_core::grpc;
use p2p_stack::Client;
#[cfg(feature = "batman")]
use p2p_stack::DebugClientCommand;
use tokio::net::UnixListener;
#[cfg(feature = "batman")]
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server as TonicServer;
use ulid::Ulid;

#[cfg(feature = "batman")]
use crate::debug::DebugServer;
pub use crate::{db::DbClient, scripting::ScriptingClient};
use crate::{
    db::DbServer, dht::DhtServer, discovery::DiscoveryServer, file_transfer::FileTransferServer,
    gossipsub::GossipSubServer, req_resp::ReqRespServer, scripting::ScriptingServer,
};

mod db;
#[cfg(feature = "batman")]
mod debug;
mod dht;
mod discovery;
mod file_transfer;
mod gossipsub;
mod req_resp;
mod scripting;

pub const CONTAINER_SHARED_DIR: &str = "/p2p/shared";

type TonicResult<T> = tonic::Result<tonic::Response<T>>;

type ServerStream<T> = Pin<Box<dyn Stream<Item = tonic::Result<T>> + Send>>;

#[cfg(feature = "batman")]
pub type DebugCommandSender = mpsc::Sender<DebugClientCommand>;

#[cfg(not(feature = "batman"))]
pub type DebugCommandSender = ();

pub struct Bridge<Db, Scripting> {
    pub client: BridgeClient<Db, Scripting>,
    pub cancellation_token: CancellationToken,
    pub ulid: Ulid,
    pub socket_path: PathBuf,
    pub shared_dir_path: PathBuf,
}

impl<Db: DbClient, Scripting: ScriptingClient> Bridge<Db, Scripting> {
    pub async fn new(
        client: Client,
        db_client: Db,
        mut base_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        scripting_client: Scripting,
    ) -> io::Result<Self> {
        let ulid = Ulid::new();
        base_path.push(ulid.to_string());

        tracing::debug!(id=%ulid, "Creating bridge with path {}", base_path.display());

        tokio::fs::create_dir_all(&base_path).await?;

        let socket_path = base_path.join("bridge.sock");
        let shared_dir_path = base_path.join("shared");

        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        if !shared_dir_path.exists() {
            tokio::fs::create_dir(&shared_dir_path).await?;
        }

        let socket = UnixListener::bind(&socket_path)?;
        let socket_stream = UnixListenerStream::new(socket);

        let cancellation_token = CancellationToken::new();

        let client = BridgeClient {
            client,
            db_client,
            cancellation_token: cancellation_token.clone(),
            ulid,
            base_path,
            shared_dir_path: shared_dir_path.clone(),
            socket_stream,
            #[cfg(feature = "batman")]
            debug_command_sender,
            scripting_client,
        };

        Ok(Self {
            client,
            cancellation_token,
            ulid,
            socket_path,
            shared_dir_path,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Tonic error: {0}")]
    Tonic(#[from] tonic::transport::Error),
}

pub struct BridgeClient<Db, Scripting> {
    client: Client,
    db_client: Db,
    cancellation_token: CancellationToken,
    ulid: Ulid,
    base_path: PathBuf,
    shared_dir_path: PathBuf,
    socket_stream: UnixListenerStream,
    #[cfg(feature = "batman")]
    debug_command_sender: mpsc::Sender<DebugClientCommand>,
    scripting_client: Scripting,
}

impl<Db: DbClient, Scripting: ScriptingClient> BridgeClient<Db, Scripting> {
    pub async fn run(self) -> Result<(), Error> {
        let db = DbServer::new(self.db_client);
        let dht = DhtServer::new(self.client.clone());
        let discovery = DiscoveryServer::new(self.client.clone());
        let file_transfer = FileTransferServer::new(self.client.clone(), self.shared_dir_path);
        let gossipsub = GossipSubServer::new(self.client.clone());
        let req_resp = ReqRespServer::new(self.client);
        let scripting = ScriptingServer::new(self.scripting_client, self.ulid);

        let router = TonicServer::builder()
            .add_service(grpc::db_server::DbServer::new(db))
            .add_service(grpc::dht_server::DhtServer::new(dht))
            .add_service(grpc::discovery_server::DiscoveryServer::new(discovery))
            .add_service(grpc::file_transfer_server::FileTransferServer::new(
                file_transfer,
            ))
            .add_service(grpc::gossip_sub_server::GossipSubServer::new(gossipsub))
            .add_service(grpc::req_resp_server::ReqRespServer::new(req_resp))
            .add_service(grpc::scripting_server::ScriptingServer::new(scripting));

        #[cfg(feature = "batman")]
        let debug = DebugServer::new(self.debug_command_sender);

        #[cfg(feature = "batman")]
        let router = router.add_service(grpc::debug_server::DebugServer::new(debug));

        tracing::debug!(id=%self.ulid, "Starting bridge");

        router
            .serve_with_incoming_shutdown(self.socket_stream, self.cancellation_token.cancelled())
            .await?;

        tokio::fs::remove_dir_all(self.base_path).await?;

        tracing::debug!(id=%self.ulid, "Bridge stopped");

        Ok(())
    }
}
