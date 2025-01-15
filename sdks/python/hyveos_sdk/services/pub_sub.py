from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import PubSubStub
from ..protocol.bridge_pb2 import (
    Data,
    PubSubMessage,
    PubSubRecvMessage,
    Topic,
)
from .stream import ManagedStream
from .util import enc


class PubSubService:
    """
    A handle to the pub-sub service.

    Exposes methods to interact with the pub-sub service, like for subscribing to topics and
    publishing messages.
    """

    def __init__(self, conn: Channel):
        self.stub = PubSubStub(conn)

    def subscribe(self, topic: str) -> ManagedStream[PubSubRecvMessage]:
        """
        Subscribes to a topic and returns a stream of messages published to that topic.

        Parameters
        ----------
        topic : str
            The topic to subscribe to

        Returns
        -------
        stream : ManagedStream[GossipSubRecvMessage]
            Stream of received messages from the pub-sub topic
        """
        gossip_sub_recv_messages_stream = self.stub.Subscribe(Topic(topic=topic))
        return ManagedStream(gossip_sub_recv_messages_stream)

    async def publish(self, data: str | bytes, topic: str) -> bytes:
        """
        Publishes a message to a topic.

        Parameters
        ----------
        data : str | bytes
            Data to publish
        topic : str
            Topic to publish the data into

        Returns
        -------
        gossip_sub_message_id : bytes
            ID of the sent message
        """

        send_data = Data(data=enc(data))

        gossip_sub_message = PubSubMessage(data=send_data, topic=Topic(topic=topic))
        gossip_sub_message_id = await self.stub.Publish(gossip_sub_message)

        return gossip_sub_message_id.id
