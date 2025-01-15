from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import ReqRespStub
from ..protocol.bridge_pb2 import (
    Data,
    Peer,
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
    A handle to the request-response service.

    Exposes methods to interact with the request-response service, like for sending and receiving
    requests, and for sending responses.
    """

    def __init__(self, conn: Channel):
        self.stub = ReqRespStub(conn)

    async def send_request(
        self, peer_id: str, data: str | bytes, topic: Optional[str] = None
    ) -> Response:
        """
        Sends a request with an optional topic to a peer and returns the response.

        The peer must be subscribed to the topic in order to receive the request.
        If `topic` is `None`, the peer must be subscribed to `None` as well.

        Parameters
        ----------
        peer_id : str
            The peer_id of the target
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
            optional_topic = OptionalTopic(topic=Topic(topic=topic))

        send_data = Data(data=enc(data))

        message = Message(data=send_data, topic=optional_topic)
        send_request = SendRequest(peer=Peer(peer_id=peer_id), msg=message)

        response = await self.stub.Send(send_request)

        return response

    def receive(
        self,
        query: Optional[str] = None,
        regex: bool = False,
    ) -> ManagedStream[RecvRequest]:
        """
        Subscribes to a topic and returns a stream of received requests.

        Parameters
        ----------
        query : str, optional
            Either a topic subscribed to or a regex that describes topics if this argument is specified (default: None)
        regex : bool
            Query is specified as a regex, not a single `topic` string (default: False)

        Returns
        -------
        stream : ManagedStream[RecvRequest]
            Stream of received requests from the specified topic
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
        """

        if error is not None:
            response = Response(error=error)
        else:
            response = Response(data=Data(data=enc(data)))

        send_response = SendResponse(seq=seq, response=response)
        await self.stub.Respond(send_response)
