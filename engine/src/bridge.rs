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

mod dht;
mod discovery;
mod file_transfer;
mod gossipsub;
mod req_resp;

mod script {
    #![allow(clippy::pedantic)]
    tonic::include_proto!("script");
}

type TonicResult<T> = tonic::Result<tonic::Response<T>>;

type ServerStream<T> = Pin<Box<dyn Stream<Item = tonic::Result<T>> + Send>>;

pub struct BuiltBridge {
    pub bridge: Bridge,
    pub ulid: Ulid,
    pub socket_path: PathBuf,
    pub shared_dir_path: PathBuf,
}

pub struct Bridge {
    client: Client,
    socket_stream: UnixListenerStream,
    shared_dir_path: PathBuf,
}

impl Bridge {
    pub async fn build(client: Client, mut base_path: PathBuf) -> io::Result<BuiltBridge> {
        let ulid = Ulid::new();
        base_path.push(ulid.to_string());

        tokio::fs::create_dir_all(&base_path).await?;

        let socket_path = base_path.join("bridge.sock");
        let shared_dir_path = base_path.join("shared");

        let socket = UnixListener::bind(&socket_path)?;
        let socket_stream = UnixListenerStream::new(socket);

        let bridge = Self {
            client,
            socket_stream,
            shared_dir_path: shared_dir_path.clone(),
        };

        Ok(BuiltBridge {
            bridge,
            ulid,
            socket_path,
            shared_dir_path,
        })
    }

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

        router
            .serve_with_incoming(self.socket_stream)
            .await
            .expect("GRPC server failed");
    }
}
