use futures::stream::Stream;
use std::{io, path::PathBuf, pin::Pin};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server as TonicServer;
use ulid::Ulid;

use self::{
    dht::DhtServer, discovery::DiscoveryServer, file_transfer::FileTransferServer,
    gossipsub::GossipSubServer, req_resp::ReqRespServer,
};
use crate::p2p::Client;

#[cfg(feature = "batman")]
use tokio::sync::mpsc;

#[cfg(feature = "batman")]
use self::debug::DebugServer;
#[cfg(feature = "batman")]
use crate::debug::Command as DebugCommand;

mod dht;
mod discovery;
mod file_transfer;
mod gossipsub;
mod req_resp;

#[cfg(feature = "batman")]
mod debug;

mod script {
    #![allow(clippy::pedantic)]
    tonic::include_proto!("script");
}

type TonicResult<T> = tonic::Result<tonic::Response<T>>;

type ServerStream<T> = Pin<Box<dyn Stream<Item = tonic::Result<T>> + Send>>;

#[cfg(feature = "batman")]
type DebugCommandSender = mpsc::Sender<DebugCommand>;

#[cfg(not(feature = "batman"))]
type DebugCommandSender = ();

pub struct Bridge {
    pub client: BridgeClient,
    pub ulid: Ulid,
    pub socket_path: PathBuf,
    pub shared_dir_path: PathBuf,
}

impl Bridge {
    pub async fn new(
        client: Client,
        mut base_path: PathBuf,
        #[cfg_attr(not(feature = "batman"), allow(unused_variables))]
        debug_command_sender: DebugCommandSender,
    ) -> io::Result<Self> {
        let ulid = Ulid::new();
        base_path.push(ulid.to_string());

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

        let client = BridgeClient {
            client,
            socket_stream,
            shared_dir_path: shared_dir_path.clone(),
            #[cfg(feature = "batman")]
            debug_command_sender,
        };

        Ok(Self {
            client,
            ulid,
            socket_path,
            shared_dir_path,
        })
    }
}

pub struct BridgeClient {
    client: Client,
    socket_stream: UnixListenerStream,
    shared_dir_path: PathBuf,
    #[cfg(feature = "batman")]
    debug_command_sender: mpsc::Sender<DebugCommand>,
}

impl BridgeClient {
    pub async fn run(self) {
        let dht = DhtServer::new(self.client.clone());
        let discovery = DiscoveryServer::new(self.client.clone());
        let file_transfer = FileTransferServer::new(self.client.clone(), self.shared_dir_path);
        let gossipsub = GossipSubServer::new(self.client.clone());
        let req_resp = ReqRespServer::new(self.client.clone());

        let router = TonicServer::builder()
            .add_service(script::dht_server::DhtServer::new(dht))
            .add_service(script::discovery_server::DiscoveryServer::new(discovery))
            .add_service(script::file_transfer_server::FileTransferServer::new(
                file_transfer,
            ))
            .add_service(script::gossip_sub_server::GossipSubServer::new(gossipsub))
            .add_service(script::req_resp_server::ReqRespServer::new(req_resp));

        #[cfg(feature = "batman")]
        let debug = DebugServer::new(self.debug_command_sender);

        #[cfg(feature = "batman")]
        let router = router.add_service(script::debug_server::DebugServer::new(debug));

        router
            .serve_with_incoming(self.socket_stream)
            .await
            .expect("GRPC server failed");
    }
}
