from ..protocol.script_pb2_grpc import ReqRespStub
from ..protocol.script_pb2 import (
    Response,
    OptionalTopic,
    OptionalTopicQuery,
    TopicQuery,
    Topic,
    Message,
    SendRequest,
    RecvRequest,
    SendResponse,
)
from stream import ManagedStream
from util import enc


class RequestResponseService:
    """
    Direct peer-to-peer message exchange (Unicast)
    """

    def __init__(self, conn):
        self.stub = ReqRespStub(conn)

    async def send_request(
        self, peer_id: str, data: str | bytes, topic: str = None
    ) -> Response:
        """
        Send a request with an optional topic to a peer and await a response

        Parameters
        ----------
        peer_id : str
            Peer, the peer_id of the target
        data : str | bytes
            Data to send
        topic : str, optional
            Topic the peer should be subscribed to if this argument is specified (default: None)

        Returns
        -------
        response : Response
            Reponse from Peer `peer_id` to the sent request, awaited
        """

        optional_topic = OptionalTopic()
        if topic is not None:
            optional_topic.topic = Topic(topic)

        send_data = enc(data)

        message = Message(send_data, optional_topic)
        send_request = SendRequest(peer_id, message)

        response = await self.stub.Send(send_request)

        return response

    async def receive(
        self, query: str = None, haveTopicQuery: bool = False, haveRegex: bool = False
    ) -> ManagedStream[RecvRequest]:
        """
        Receive requests from peers that either have no topic or have a topic that has been subscribed to

        Parameters
        ----------
        query : str, optional
            Either a topic subscribed to or a regex that describes topics if this argument is specified (default: None)
        haveTopicQuery : bool
            Receive from a topic described by the query, is set in accordance with `query` (default: False)
        haveRegex : bool
            Query is specified as a regex, not a single `topic` string (default: False)

        Returns
        -------
        stream : ManagedStream[RecvRequest]
            Iterator to handle the stream of RecvRequests
        """

        assert (query is None and haveTopicQuery is False and haveRegex is None) or (
            query is not None and haveTopicQuery is True
        ), 'having a topicQuery and providing a query have to match'

        optional_topic_query = OptionalTopicQuery()

        if haveTopicQuery:
            topic_query = TopicQuery()
            if not haveRegex:
                topic_struct = Topic(query)
                topic_query.topic = topic_struct
            else:
                regex = query
                topic_query.regex = regex
            optional_topic_query.query = topic_query

        stream = self.stub.Recv(optional_topic_query)
        return ManagedStream(stream)

    async def respond(self, seq: int, data: str | bytes, error: str = None) -> None:
        """
        Respond to a request received from receive()

        Parameters
        ----------
        seq : int
            Sequence number for request-response matching
        data : str | bytes
            Reponse message data. If error is specified, this won't reach the peer
        error : str
            Respond with an error message if an error occurred (default:  None)

        Returns
        -------
        None
        """

        response = Response()
        if error is not None:
            response.error = error
        else:
            response.data = enc(data)

        send_response = SendResponse(seq, response)
        await self.stub.Respond(send_response)
