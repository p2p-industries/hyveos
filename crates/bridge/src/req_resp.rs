use drop_stream::DropStream;
use futures::StreamExt as _;
use p2p_stack::{
    req_resp::{Request, Response, TopicQuery},
    Client,
};
use regex::Regex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{
    script::{self, req_resp_server::ReqResp},
    ServerStream, TonicResult,
};

impl From<Request> for script::Message {
    fn from(request: Request) -> Self {
        Self {
            data: request.data,
            topic: script::OptionalTopic {
                topic: request.topic.as_ref().map(|topic| script::Topic {
                    topic: topic.to_string(),
                }),
            },
        }
    }
}

impl From<script::Message> for Request {
    fn from(message: script::Message) -> Self {
        Self {
            data: message.data,
            topic: message.topic.topic.map(|topic| topic.topic.into()),
        }
    }
}

impl From<Response> for script::Response {
    fn from(response: Response) -> Self {
        Self {
            response: Some(match response {
                Response::Data(data) => script::response::Response::Data(data),
                Response::Error(e) => script::response::Response::Error(e.to_string()),
            }),
        }
    }
}

impl TryFrom<script::Response> for Response {
    type Error = Status;

    fn try_from(response: script::Response) -> Result<Self, Status> {
        Ok(
            match response
                .response
                .ok_or(Status::invalid_argument("Response is missing"))?
            {
                script::response::Response::Data(data) => Self::Data(data),
                script::response::Response::Error(e) => Self::Error(e.into()),
            },
        )
    }
}

impl TryFrom<script::TopicQuery> for TopicQuery {
    type Error = Status;

    fn try_from(query: script::TopicQuery) -> Result<Self, Status> {
        Ok(
            match query
                .query
                .ok_or(Status::invalid_argument("Query is missing"))?
            {
                script::topic_query::Query::Regex(regex) => Self::Regex(
                    Regex::new(&regex)
                        .map_err(|e| Status::invalid_argument(format!("Invalid regex: {e}")))?,
                ),
                script::topic_query::Query::Topic(topic) => Self::String(topic.topic.into()),
            },
        )
    }
}

pub struct ReqRespServer {
    client: Client,
}

impl ReqRespServer {
    pub fn new(client: Client) -> Self {
        Self { client }
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

        tracing::debug!(?request, "Received send request");

        let peer_id = request
            .peer_id
            .parse()
            .map_err(|e| Status::invalid_argument(format!("Failed to parse peer id: {e}")))?;

        let message = request.msg;

        self.client
            .req_resp()
            .send_request(peer_id, message.into())
            .await
            .map(|res| TonicResponse::new(res.into()))
            .map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn recv(
        &self,
        request: TonicRequest<script::OptionalTopicQuery>,
    ) -> TonicResult<Self::RecvStream> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received recv request");

        let query = request.query.map(TryInto::try_into).transpose()?;

        let client = self.client.clone();

        let (id, receiver) = client
            .req_resp()
            .subscribe(query)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        let stream = ReceiverStream::new(receiver).map(|req| {
            Ok(script::RecvRequest {
                peer_id: req.peer_id.to_string(),
                msg: req.req.into(),
                seq: req.id,
            })
        });

        let drop_stream = DropStream::new(stream, move || {
            tokio::spawn(async move {
                let _ = client.req_resp().unsubscribe(id).await;
            });
        });

        Ok(TonicResponse::new(Box::pin(drop_stream)))
    }

    async fn respond(
        &self,
        request: TonicRequest<script::SendResponse>,
    ) -> TonicResult<script::Empty> {
        let response = request.into_inner();

        tracing::debug!(request=?response, "Received respond request");

        self.client
            .req_resp()
            .send_response(response.seq, response.response.try_into()?)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        Ok(TonicResponse::new(script::Empty {}))
    }
}
