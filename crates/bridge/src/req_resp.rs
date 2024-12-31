use drop_stream::DropStream;
use futures::StreamExt as _;
use hyveos_core::grpc::{self, req_resp_server::ReqResp};
use hyveos_p2p_stack::Client;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status};

use crate::{ServerStream, TonicResult};

pub struct ReqRespServer {
    client: Client,
}

impl ReqRespServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait] // TODO: rewrite when https://github.com/hyperium/tonic/pull/1697 is merged
impl ReqResp for ReqRespServer {
    type RecvStream = ServerStream<grpc::RecvRequest>;

    async fn send(&self, request: TonicRequest<grpc::SendRequest>) -> TonicResult<grpc::Response> {
        let request = request.into_inner();

        tracing::debug!(?request, "Received send request");

        let grpc::SendRequest { peer, msg } = request;

        self.client
            .req_resp()
            .send_request(peer.try_into()?, msg.into())
            .await
            .map(|res| TonicResponse::new(res.into()))
            .map_err(|e| Status::internal(format!("{e:?}")))
    }

    async fn recv(
        &self,
        request: TonicRequest<grpc::OptionalTopicQuery>,
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

        let stream = ReceiverStream::new(receiver).map(Into::into).map(Ok);

        let drop_stream = DropStream::new(stream, move || {
            tokio::spawn(async move {
                let _ = client.req_resp().unsubscribe(id).await;
            });
        })
        .boxed();

        Ok(TonicResponse::new(drop_stream))
    }

    async fn respond(&self, request: TonicRequest<grpc::SendResponse>) -> TonicResult<grpc::Empty> {
        let response = request.into_inner();

        tracing::debug!(request=?response, "Received respond request");

        self.client
            .req_resp()
            .send_response(response.seq, response.response.try_into()?)
            .await
            .map_err(|e| Status::internal(format!("{e:?}")))?;

        Ok(TonicResponse::new(grpc::Empty {}))
    }
}
