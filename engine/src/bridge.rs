use futures::{
    future,
    stream::{Stream, TryStreamExt as _},
};
use std::{path::Path, pin::Pin};
use tokio::net::UnixListener;
use tokio_stream::wrappers::{BroadcastStream, UnixListenerStream};
use tonic::{transport::Server as TonicServer, Request, Response, Result, Status};

use self::script::{discovery_server::Discovery, req_resp_server::ReqResp};
use crate::p2p::{neighbours::Event, Client};

mod script {
    tonic::include_proto!("script");
}

type ServerStream<T> = Pin<Box<dyn Stream<Item = Result<T>> + Send>>;

struct ReqRespServer {
    client: Client,
}

impl ReqRespServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl ReqResp for ReqRespServer {
    type RecvStream = ServerStream<script::Request>;

    async fn send(&self, request: Request<script::Request>) -> Result<Response<script::Response>> {
        let request = request.into_inner();
        let peer_id = request
            .peer_id
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Failed to parse peer id: {e}")))?;

        self.client
            .req_resp()
            .send_request(peer_id, request.data)
            .await
            .map(|res| Response::new(script::Response { data: res }))
            .map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn recv(&self, _request: Request<script::Empty>) -> Result<Response<Self::RecvStream>> {
        let sub = self
            .client
            .req_resp()
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let stream = BroadcastStream::new(sub)
            .map_ok(|req| script::Request {
                peer_id: req.peer_id.to_string(),
                data: req.data,
                seq: Some(req.id),
            })
            .map_err(|e| Status::internal(e.to_string()));

        Ok(Response::new(Box::pin(stream)))
    }

    async fn respond(
        &self,
        request: Request<script::SendResponse>,
    ) -> Result<Response<script::Empty>> {
        let response = request.into_inner();

        self.client
            .req_resp()
            .send_response(response.seq, response.response.data)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        Ok(Response::new(script::Empty {}))
    }
}

struct DiscoveryServer {
    client: Client,
}

impl DiscoveryServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl Discovery for DiscoveryServer {
    type DiscoverStream = ServerStream<script::Discovered>;

    async fn discover(
        &self,
        _request: Request<script::Empty>,
    ) -> Result<Response<Self::DiscoverStream>> {
        let neighbours = self.client.neighbours();
        let sub = neighbours
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let stream = BroadcastStream::new(sub)
            .try_filter_map(|event| {
                future::ready(Ok(match event.as_ref() {
                    Event::ResolvedNeighbour { neighbour, .. } => Some(script::Discovered {
                        peer_id: neighbour.peer_id.to_string(),
                    }),
                    _ => None,
                }))
            })
            .map_err(|e| Status::internal(e.to_string()));

        Ok(Response::new(Box::pin(stream)))
    }
}

pub struct Bridge {
    client: Client,
}

impl Bridge {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn run(self) {
        let path = "/var/run/p2p-bridge.sock";

        tokio::fs::create_dir_all(Path::new(path).parent().unwrap())
            .await
            .expect("Failed to create socket directory");

        let req_resp = ReqRespServer::new(self.client.clone());
        let discovery = DiscoveryServer::new(self.client.clone());

        let uds = UnixListener::bind(path).expect("Failed to bind to UDS");
        let uds_stream = UnixListenerStream::new(uds);

        TonicServer::builder()
            .add_service(script::req_resp_server::ReqRespServer::new(req_resp))
            .add_service(script::discovery_server::DiscoveryServer::new(discovery))
            .serve_with_incoming(uds_stream)
            .await
            .expect("GRPC server failed");
    }
}
