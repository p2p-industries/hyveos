import protocol.script_pb2 as script_pb2
import protocol.script_pb2_grpc as script_pb2_grpc

from stream import ManagedStream


class RequestResponseService:
    """
    Direct peer-to-peer message exchange
    """

    def __init__(self, conn):
        self.stub = script_pb2_grpc.ReqRespStub(conn)

    async def send_request(
        self, peer_id: str, data: str | bytes, topic: str = None
    ) -> script_pb2.Response:
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
        response : script_pb2.Response
            Reponse from Peer `peer_id` to the sent request, awaited
        """

        optional_topic = script_pb2.OptionalTopic()
        if topic is not None:
            script_pb2.OptionalTopic(script_pb2.Topic(topic))

        send_data = data
        if isinstance(data) == str:
            send_data = data.encode('utf-8')

        message = script_pb2.Message(send_data, optional_topic)
        send_request = script_pb2.SendRequest(peer_id, message)

        response = await self.stub.Send(send_request)

        return response

    async def receive(
        self, query: str = None, haveTopicQuery: bool = False, haveRegex: bool = False
    ) -> ManagedStream:
        """
        Receive requests from peers that either have no topic or have a topic that has been subscribed to

        Parameters
        ----------
        query : str, optional
            Either a topic subscribed to or a regex that describes topics if this argument is specified (default: None)
        haveTopicQuery : bool
            Receive from a topic descirbed by the query, is set in accordance with `query` (default: False)
        haveRegex : bool
            Query is specified as a regex, not a single `topic` string (default: False)

        Returns
        -------
        ManagedStream
            Iterator to handle the stream of RecvRequests
        """

        assert (query is None and haveTopicQuery is False) or (
            query is not None and haveTopicQuery is True
        ), 'having a topicQuery and providing a query have to match'

        optional_topic_query = script_pb2.OptionalTopicQuery()

        if haveTopicQuery:
            topic_query = script_pb2.TopicQuery()
            if not haveRegex:
                topic_struct = script_pb2.Topic(query)
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
            Reponse message data, if error is specified, this won't reach the peer
        error : str
            Respond with an error message if an error occurred (default:  None)

        Returns
        -------
        None
        """

        response = script_pb2.Reponse()
        if error is not None:
            response.error = error
        else:
            send_data = data
            if isinstance(data) == str:
                send_data = data.encode('utf-8')
            response.data = send_data

        send_response = script_pb2.Message(seq, response)
        self.stub.Respond(send_response)
