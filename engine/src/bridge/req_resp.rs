use std::{collections::HashSet, sync::Arc};

use futures::TryStreamExt as _;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::p2p::{req_resp::Request, Client};

use super::{
    script::{self, req_resp_server::ReqResp},
    ServerStream, TonicResult,
};

impl From<Request> for script::Message {
    fn from(request: Request) -> Self {
        Self {
            data: request.data,
            topic: request.topic.as_ref().map(|topic| script::Topic {
                topic: topic.to_string(),
            }),
        }
    }
}

impl From<script::Message> for Request {
    fn from(message: script::Message) -> Self {
        Self {
            data: message.data,
            topic: message.topic.map(|topic| topic.topic.into()),
        }
    }
}

pub struct ReqRespServer {
    client: Client,
    subscribed_topics: Arc<RwLock<HashSet<Arc<str>>>>,
}

impl ReqRespServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            subscribed_topics: Arc::default(),
        }
    }
}

#[tonic::async_trait]
impl ReqResp for ReqRespServer {
    type RecvStream = ServerStream<script::RecvRequest>;

    async fn send(
        &self,
        request: TonicRequest<script::SendRequest>,
    ) -> TonicResult<script::Response> {
        let request = request.into_inner();
        let peer_id = request
            .peer_id
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Failed to parse peer id: {e}")))?;

        let message = request.msg;

        self.client
            .req_resp()
            .send_request(peer_id, message.into())
            .await
            .map(|res| TonicResponse::new(script::Response { data: res }))
            .map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn recv(&self, _request: TonicRequest<script::Empty>) -> TonicResult<Self::RecvStream> {
        let sub = self
            .client
            .req_resp()
            .subscribe()
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let subcribed_topics = self.subscribed_topics.clone();

        let stream = BroadcastStream::new(sub)
            .try_filter(move |req| {
                let topic = req.req.topic.clone();
                let subcribed_topics = subcribed_topics.clone();
                async move {
                    if let Some(topic) = topic {
                        subcribed_topics.read().await.contains(&topic)
                    } else {
                        true
                    }
                }
            })
            .map_ok(|req| script::RecvRequest {
                peer_id: req.peer_id.to_string(),
                msg: req.req.into(),
                seq: req.id,
            })
            .map_err(|e| Status::internal(e.to_string()));

        Ok(TonicResponse::new(Box::pin(stream)))
    }

    async fn respond(
        &self,
        request: TonicRequest<script::SendResponse>,
    ) -> TonicResult<script::Empty> {
        let response = request.into_inner();

        self.client
            .req_resp()
            .send_response(response.seq, response.response.data)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        Ok(TonicResponse::new(script::Empty {}))
    }

    async fn subscribe(&self, request: TonicRequest<script::Topic>) -> TonicResult<script::Empty> {
        let topic = request.into_inner().topic;

        self.subscribed_topics.write().await.insert(topic.into());

        Ok(TonicResponse::new(script::Empty {}))
    }

    async fn unsubscribe(
        &self,
        request: TonicRequest<script::Topic>,
    ) -> TonicResult<script::Empty> {
        let topic = request.into_inner().topic;

        self.subscribed_topics
            .write()
            .await
            .remove(&Arc::from(topic));

        Ok(TonicResponse::new(script::Empty {}))
    }
}
