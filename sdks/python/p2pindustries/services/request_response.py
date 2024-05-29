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
from .stream import ManagedStream
from .util import enc

from typing import Optional


class RequestResponseService:
    """
    Direct peer-to-peer message exchange (Unicast)
    """

    def __init__(self, conn):
        self.stub = ReqRespStub(conn)

    async def send_request(
        self, peer_id: str, data: str | bytes, topic: Optional[str] = None
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
            optional_topic.topic = Topic(topic=topic)

        send_data = enc(data)

        message = Message(data=send_data, topic=optional_topic)
        send_request = SendRequest(peer_id=peer_id, msg=message)

        response = await self.stub.Send(send_request)

        return response

    def receive(
        self,
        query: Optional[str] = None,
        regex: bool = False,
    ) -> ManagedStream[RecvRequest]:
        """
        Receive requests from peers that either have no topic or have a topic that has been subscribed to

        Parameters
        ----------
        query : str, optional
            Either a topic subscribed to or a regex that describes topics if this argument is specified (default: None)
        regex : bool
            Query is specified as a regex, not a single `topic` string (default: False)

        Returns
        -------
        stream : ManagedStream[RecvRequest]
            Iterator to handle the stream of RecvRequests
        """

        optional_topic_query = OptionalTopicQuery()

        if query is not None:
            if regex:
                optional_topic_query = OptionalTopicQuery(query=TopicQuery(regex=query))
            else:
                optional_topic_query = OptionalTopicQuery(
                    query=TopicQuery(topic=Topic(topic=query))
                )

        stream = self.stub.Recv(optional_topic_query)
        return ManagedStream(stream)

    async def respond(
        self, seq: int, data: str | bytes, error: Optional[str] = None
    ) -> None:
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

        if error is not None:
            response = Response(error=error)
        else:
            response = Response(data=enc(data))

        send_response = SendResponse(seq=seq, response=response)
        await self.stub.Respond(send_response)
