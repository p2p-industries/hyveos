import protocol.script_pb2 as script_pb2
import protocol.script_pb2_grpc as script_pb2_grpc

from stream import ManagedStream


class GossipSubService:
    """
    Subscribing or Publishing into a Topic
    """

    def __init__(self, conn):
        self.stub = script_pb2_grpc.GossipSubStub(conn)

    async def subscribe(self, topic: str) -> ManagedStream:
        """
        Subscribe to a GossipSub Topic to receive messages published in that topic

        Parameters
        ----------
        topic : str
            Topic to subscribe to

        Returns
        -------
        straem : ManagedStream
            Stream of received messages from a GossipSub topic
        """

        gossip_sub_recv_messages_stream = self.stub.Subscribe(script_pb2.Topic(topic))
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

        send_data = data
        if isinstance(data) == str:
            send_data = data.encode('utf-8')

        gossip_sub_message = script_pb2.GossipSubMessage(
            send_data, script_pb2.Topic(topic)
        )
        gossip_sub_message_id = await self.stub.Publish(gossip_sub_message)

        return gossip_sub_message_id.id
