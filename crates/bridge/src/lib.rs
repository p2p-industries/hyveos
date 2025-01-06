#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

#[cfg(feature = "network")]
use std::net::SocketAddr;
use std::{io, os::unix::fs::PermissionsExt as _, path::PathBuf};

use futures::stream::BoxStream;
use hyveos_core::grpc;
use hyveos_p2p_stack::Client;
#[cfg(feature = "batman")]
use hyveos_p2p_stack::DebugClientCommand;
#[cfg(feature = "network")]
use tokio::net::TcpListener;
use tokio::net::UnixListener;
#[cfg(feature = "batman")]
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;
#[cfg(feature = "network")]
use tonic::service::Routes as TonicRoutes;
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

pub const CONTAINER_SHARED_DIR: &str = "/hyveos/shared";

type TonicResult<T> = tonic::Result<tonic::Response<T>>;

type ServerStream<T> = BoxStream<'static, tonic::Result<T>>;

#[cfg(feature = "batman")]
pub type DebugCommandSender = mpsc::Sender<DebugClientCommand>;

#[cfg(not(feature = "batman"))]
pub type DebugCommandSender = ();

enum Connection {
    Local(UnixListener),
    #[cfg(feature = "network")]
    Network(TcpListener),
}

pub struct Bridge<Db, Scripting> {
    pub client: BridgeClient<Db, Scripting>,
    pub cancellation_token: CancellationToken,
}

impl<Db: DbClient, Scripting: ScriptingClient> Bridge<Db, Scripting> {
    pub async fn new(
        client: Client,
        db_client: Db,
        base_path: PathBuf,
        socket_path: PathBuf,
        shared_dir_path: Option<PathBuf>,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        scripting_client: Scripting,
    ) -> io::Result<Self> {
        tokio::fs::create_dir_all(&base_path).await?;

        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        if let Some(shared_dir_path) = &shared_dir_path {
            if !shared_dir_path.exists() {
                tokio::fs::create_dir_all(&shared_dir_path).await?;
            }
        }

        let socket = UnixListener::bind(&socket_path)?;

        let mut permissions = socket_path.metadata()?.permissions();
        permissions.set_mode(0o775);
        tokio::fs::set_permissions(&socket_path, permissions).await?;

        let cancellation_token = CancellationToken::new();

        let client = BridgeClient {
            client,
            db_client,
            cancellation_token: cancellation_token.clone(),
            ulid: None,
            base_path,
            shared_dir_path,
            connection: Connection::Local(socket),
            #[cfg(feature = "batman")]
            debug_command_sender,
            scripting_client,
        };

        Ok(Self {
            client,
            cancellation_token,
        })
    }
}

#[cfg(feature = "network")]
pub struct NetworkBridge<Db, Scripting> {
    pub client: BridgeClient<Db, Scripting>,
    pub cancellation_token: CancellationToken,
}

#[cfg(feature = "network")]
impl<Db: DbClient, Scripting: ScriptingClient> NetworkBridge<Db, Scripting> {
    pub async fn new(
        client: Client,
        db_client: Db,
        base_path: PathBuf,
        socket_addr: SocketAddr,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        scripting_client: Scripting,
    ) -> io::Result<Self> {
        tokio::fs::create_dir_all(&base_path).await?;

        let socket = TcpListener::bind(socket_addr).await?;

        let cancellation_token = CancellationToken::new();

        let client = BridgeClient {
            client,
            db_client,
            cancellation_token: cancellation_token.clone(),
            ulid: None,
            base_path,
            shared_dir_path: None,
            connection: Connection::Network(socket),
            #[cfg(feature = "batman")]
            debug_command_sender,
            scripting_client,
        };

        Ok(Self {
            client,
            cancellation_token,
        })
    }
}

pub struct ScriptingBridge<Db, Scripting> {
    pub client: BridgeClient<Db, Scripting>,
    pub cancellation_token: CancellationToken,
    pub ulid: Ulid,
    pub socket_path: PathBuf,
    pub shared_dir_path: PathBuf,
}

impl<Db: DbClient, Scripting: ScriptingClient> ScriptingBridge<Db, Scripting> {
    pub async fn new(
        client: Client,
        db_client: Db,
        mut base_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        scripting_client: Scripting,
    ) -> io::Result<Self> {
        let ulid = Ulid::new();
        base_path.push(ulid.to_string());

        tracing::debug!(id=%ulid, path=%base_path.display(), "Creating bridge");

        let socket_path = base_path.join("bridge.sock");
        let shared_dir_path = base_path.join("files");

        let Bridge {
            mut client,
            cancellation_token,
        } = Bridge::new(
            client,
            db_client,
            base_path,
            socket_path.clone(),
            Some(shared_dir_path.clone()),
            #[cfg(feature = "batman")]
            debug_command_sender,
            scripting_client,
        )
        .await?;

        client.ulid = Some(ulid);

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
    ulid: Option<Ulid>,
    base_path: PathBuf,
    shared_dir_path: Option<PathBuf>,
    connection: Connection,
    #[cfg(feature = "batman")]
    debug_command_sender: mpsc::Sender<DebugClientCommand>,
    scripting_client: Scripting,
}

macro_rules! build_tonic {
    (
        $tonic:expr,
        $db:ident,
        $dht:ident,
        $discovery:ident,
        $gossipsub:ident,
        $req_resp:ident,
        $scripting:ident,
        $debug:ident
    ) => {{
        let tmp = $tonic
            .add_service(grpc::db_server::DbServer::new($db))
            .add_service(grpc::dht_server::DhtServer::new($dht))
            .add_service(grpc::discovery_server::DiscoveryServer::new($discovery))
            .add_service(grpc::gossip_sub_server::GossipSubServer::new($gossipsub))
            .add_service(grpc::req_resp_server::ReqRespServer::new($req_resp))
            .add_service(grpc::scripting_server::ScriptingServer::new($scripting));

        #[cfg(feature = "batman")]
        let tmp = tmp.add_service(grpc::debug_server::DebugServer::new($debug));

        tmp
    }};
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

        #[cfg(feature = "batman")]
        let debug = DebugServer::new(self.debug_command_sender);

        tracing::debug!(id=?self.ulid, "Starting bridge");

        match self.connection {
            Connection::Local(socket) => {
                let router = build_tonic!(
                    TonicServer::builder(),
                    db,
                    dht,
                    discovery,
                    gossipsub,
                    req_resp,
                    scripting,
                    debug
                )
                .add_service(grpc::file_transfer_server::FileTransferServer::new(
                    file_transfer,
                ));

                let socket_stream = UnixListenerStream::new(socket);
                router
                    .serve_with_incoming_shutdown(
                        socket_stream,
                        self.cancellation_token.cancelled(),
                    )
                    .await?;
            }
            #[cfg(feature = "network")]
            Connection::Network(listener) => {
                let router = axum::Router::new()
                    .route(
                        "/file-transfer/publish-file/:file_name",
                        axum::routing::post(FileTransferServer::publish_file_http),
                    )
                    .route(
                        "/file-transfer/get-file",
                        axum::routing::get(FileTransferServer::get_file_http),
                    )
                    .with_state(file_transfer);

                let tonic_routes = build_tonic!(
                    TonicRoutes::from(router),
                    db,
                    dht,
                    discovery,
                    gossipsub,
                    req_resp,
                    scripting,
                    debug
                );

                let router = tonic_routes.into_axum_router();

                axum::serve(listener, router)
                    .with_graceful_shutdown(self.cancellation_token.cancelled_owned())
                    .await?;
            }
        }

        tokio::fs::remove_dir_all(self.base_path).await?;

        tracing::debug!(id=?self.ulid, "Bridge stopped");

        Ok(())
    }
}
