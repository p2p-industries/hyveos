from ..protocol.script_pb2_grpc import GossipSubStub
from ..protocol.script_pb2 import (
    GossipSubMessage,
    GossipSubRecvMessage,
    Topic,
)
from .stream import ManagedStream
from .util import enc


class GossipSubService:
    """
    Subscribing or Publishing into a Topic
    """

    def __init__(self, conn):
        self.stub = GossipSubStub(conn)

    async def subscribe(self, topic: str) -> ManagedStream[GossipSubRecvMessage]:
        """
        Subscribe to a GossipSub Topic to receive messages published in that topic

        Parameters
        ----------
        topic : str
            Topic to subscribe to

        Returns
        -------
        stream : ManagedStream[GossipSubRecvMessage]
            Stream of received messages from a GossipSub topic
        """

        gossip_sub_recv_messages_stream = self.stub.Subscribe(Topic(topic=topic))
        return ManagedStream(gossip_sub_recv_messages_stream)

    async def publish(self, data: str | bytes, topic: str) -> bytes:
        """
        Publish a message in a GossipSub Topic

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

        send_data = enc(data)

        gossip_sub_message = GossipSubMessage(data=send_data, topic=Topic(topic=topic))
        gossip_sub_message_id = await self.stub.Publish(gossip_sub_message)

        return gossip_sub_message_id.id
