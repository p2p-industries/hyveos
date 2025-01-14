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
pub use crate::{apps::AppsClient, local_kv::DbClient, telemetry::Telemetry};
use crate::{
    apps::AppsServer, discovery::DiscoveryServer, file_transfer::FileTransferServer, kv::KvServer,
    local_kv::LocalKvServer, neighbours::NeighboursServer, pub_sub::PubSubServer,
    req_resp::ReqRespServer,
};

mod apps;
#[cfg(feature = "batman")]
mod debug;
mod discovery;
mod file_transfer;
mod kv;
mod local_kv;
mod neighbours;
mod pub_sub;
mod req_resp;
mod telemetry;

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

pub struct Bridge<Db, Apps> {
    pub client: BridgeClient<Db, Apps>,
    pub cancellation_token: CancellationToken,
    pub shared_dir_path: PathBuf,
}

impl<Db: DbClient, Apps: AppsClient> Bridge<Db, Apps> {
    #[cfg_attr(feature = "batman", expect(clippy::too_many_arguments))]
    pub async fn new(
        client: Client,
        db_client: Db,
        base_path: PathBuf,
        socket_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        apps_client: Apps,
        is_application_bridge: bool,
        telemetry: Telemetry,
    ) -> io::Result<Self> {
        tokio::fs::create_dir_all(&base_path).await?;

        let shared_dir_path = base_path.join("files");

        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        if !shared_dir_path.exists() {
            tokio::fs::create_dir(&shared_dir_path).await?;
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
            shared_dir_path: shared_dir_path.clone(),
            connection: Connection::Local(socket),
            #[cfg(feature = "batman")]
            debug_command_sender,
            apps_client,
            is_application_bridge,
            telemetry,
        };

        Ok(Self {
            client,
            cancellation_token,
            shared_dir_path,
        })
    }
}

#[cfg(feature = "network")]
pub struct NetworkBridge<Db, Apps> {
    pub client: BridgeClient<Db, Apps>,
    pub cancellation_token: CancellationToken,
}

#[cfg(feature = "network")]
impl<Db: DbClient, Apps: AppsClient> NetworkBridge<Db, Apps> {
    pub async fn new(
        client: Client,
        db_client: Db,
        base_path: PathBuf,
        socket_addr: SocketAddr,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        apps_client: Apps,
        telemetry: Telemetry,
    ) -> io::Result<Self> {
        tokio::fs::create_dir_all(&base_path).await?;

        let shared_dir_path = base_path.join("files");

        if !shared_dir_path.exists() {
            tokio::fs::create_dir(&shared_dir_path).await?;
        }

        let socket = TcpListener::bind(socket_addr).await?;

        let cancellation_token = CancellationToken::new();

        let client = BridgeClient {
            client,
            db_client,
            cancellation_token: cancellation_token.clone(),
            ulid: None,
            base_path,
            shared_dir_path,
            connection: Connection::Network(socket),
            #[cfg(feature = "batman")]
            debug_command_sender,
            apps_client,
            is_application_bridge: false,
            telemetry,
        };

        Ok(Self {
            client,
            cancellation_token,
        })
    }
}

pub struct ApplicationBridge<Db, Apps> {
    pub client: BridgeClient<Db, Apps>,
    pub cancellation_token: CancellationToken,
    pub ulid: Ulid,
    pub socket_path: PathBuf,
    pub shared_dir_path: PathBuf,
}

impl<Db: DbClient, Apps: AppsClient> ApplicationBridge<Db, Apps> {
    pub async fn new(
        client: Client,
        db_client: Db,
        mut base_path: PathBuf,
        #[cfg(feature = "batman")] debug_command_sender: DebugCommandSender,
        apps_client: Apps,
        telemetry: Telemetry,
    ) -> io::Result<Self> {
        let ulid = Ulid::new();
        base_path.push(ulid.to_string());

        tracing::debug!(id=%ulid, path=%base_path.display(), "Creating bridge");

        let socket_path = base_path.join("bridge.sock");

        let Bridge {
            mut client,
            cancellation_token,
            shared_dir_path,
        } = Bridge::new(
            client,
            db_client,
            base_path,
            socket_path.clone(),
            #[cfg(feature = "batman")]
            debug_command_sender,
            apps_client,
            true,
            telemetry.context("scripting"),
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

pub struct BridgeClient<Db, Apps> {
    client: Client,
    db_client: Db,
    cancellation_token: CancellationToken,
    ulid: Option<Ulid>,
    base_path: PathBuf,
    shared_dir_path: PathBuf,
    connection: Connection,
    #[cfg(feature = "batman")]
    debug_command_sender: mpsc::Sender<DebugClientCommand>,
    apps_client: Apps,
    is_application_bridge: bool,
    telemetry: Telemetry,
}

macro_rules! build_tonic {
    (
        $tonic:expr,
        $apps:ident,
        $discovery:ident,
        $kv:ident,
        $local_kv:ident,
        $neighbours:ident,
        $pub_sub:ident,
        $req_resp:ident,
        $debug:ident,
        $transform:expr
    ) => {{
        let tmp = $tonic
            .add_service($transform(grpc::apps_server::AppsServer::new($apps)))
            .add_service($transform(grpc::discovery_server::DiscoveryServer::new(
                $discovery,
            )))
            .add_service($transform(grpc::kv_server::KvServer::new($kv)))
            .add_service($transform(grpc::local_kv_server::LocalKvServer::new(
                $local_kv,
            )))
            .add_service($transform(grpc::neighbours_server::NeighboursServer::new(
                $neighbours,
            )))
            .add_service($transform(grpc::pub_sub_server::PubSubServer::new(
                $pub_sub,
            )))
            .add_service($transform(grpc::req_resp_server::ReqRespServer::new(
                $req_resp,
            )));

        #[cfg(feature = "batman")]
        let tmp = tmp.add_service($transform(grpc::debug_server::DebugServer::new($debug)));

        tmp
    }};
}

impl<Db: DbClient, Apps: AppsClient> BridgeClient<Db, Apps> {
    pub async fn run(self) -> Result<(), Error> {
        let apps = AppsServer::new(
            self.apps_client,
            self.ulid,
            self.telemetry.clone().service("apps"),
        );
        let discovery = DiscoveryServer::new(
            self.client.clone(),
            self.telemetry.clone().service("discovery"),
        );
        let file_transfer = FileTransferServer::new(
            self.client.clone(),
            self.shared_dir_path,
            self.is_application_bridge,
            self.telemetry.clone().service("file_transfer"),
        );
        let kv = KvServer::new(self.client.clone(), self.telemetry.clone().service("kv"));
        let local_kv =
            LocalKvServer::new(self.db_client, self.telemetry.clone().service("local_kv"));
        let neighbours = NeighboursServer::new(
            self.client.clone(),
            self.telemetry.clone().service("neighbours"),
        );
        let pub_sub = PubSubServer::new(
            self.client.clone(),
            self.telemetry.clone().service("pub_sub"),
        );
        let req_resp = ReqRespServer::new(self.client, self.telemetry.clone().service("req_resp"));

        #[cfg(feature = "batman")]
        let debug = DebugServer::new(
            self.debug_command_sender,
            self.telemetry.clone().service("debug"),
        );

        tracing::debug!(id=?self.ulid, "Starting bridge");

        match self.connection {
            Connection::Local(socket) => {
                let router = build_tonic!(
                    TonicServer::builder(),
                    apps,
                    discovery,
                    kv,
                    local_kv,
                    neighbours,
                    pub_sub,
                    req_resp,
                    debug,
                    std::convert::identity
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
                        "/file-transfer/publish/:file_name",
                        axum::routing::post(FileTransferServer::publish_http),
                    )
                    .route(
                        "/file-transfer/get",
                        axum::routing::get(FileTransferServer::get_http),
                    )
                    .with_state(file_transfer);

                let tonic_routes = build_tonic!(
                    TonicRoutes::from(router),
                    apps,
                    discovery,
                    kv,
                    local_kv,
                    neighbours,
                    pub_sub,
                    req_resp,
                    debug,
                    tonic_web::enable
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
