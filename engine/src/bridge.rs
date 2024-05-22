use futures::stream::Stream;
use std::{path::Path, pin::Pin};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server as TonicServer;

use self::{gossipsub::GossipSubServer, req_resp::ReqRespServer};
use crate::p2p::Client;

#[cfg(feature = "batman")]
use self::discovery::DiscoveryServer;

#[cfg(feature = "batman")]
mod discovery;
mod gossipsub;
mod req_resp;

mod script {
    #![allow(clippy::pedantic)]
    tonic::include_proto!("script");
}

type TonicResult<T> = tonic::Result<tonic::Response<T>>;

type ServerStream<T> = Pin<Box<dyn Stream<Item = tonic::Result<T>> + Send>>;

pub struct Bridge {
    client: Client,
}

impl Bridge {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn run(self) {
        let path = Path::new("/home/pi/p2p-bridge.sock");

        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }

        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create socket directory");

        let gossipsub = GossipSubServer::new(self.client.clone());
        let req_resp = ReqRespServer::new(self.client.clone());

        let router = TonicServer::builder()
            .add_service(script::gossip_sub_server::GossipSubServer::new(gossipsub))
            .add_service(script::req_resp_server::ReqRespServer::new(req_resp));

        #[cfg(feature = "batman")]
        let discovery = DiscoveryServer::new(self.client.clone());
        #[cfg(feature = "batman")]
        let router = router.add_service(script::discovery_server::DiscoveryServer::new(discovery));

        let uds = UnixListener::bind(path).expect("Failed to bind to UDS");
        let uds_stream = UnixListenerStream::new(uds);

        router
            .serve_with_incoming(uds_stream)
            .await
            .expect("GRPC server failed");
    }
}
