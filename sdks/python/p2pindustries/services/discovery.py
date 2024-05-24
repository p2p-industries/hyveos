import protocol.script_pb2 as script_pb2
import protocol.script_pb2_grpc as script_pb2_grpc

from stream import ManagedStream


class DiscoveryService:
    """
    Keeping track of neighbours
    """

    def __init__(self, conn):
        self.stub = script_pb2_grpc.DiscoveryStub(conn)
        self.empty = script_pb2.Empty()

    async def discovery_events(self) -> ManagedStream:
        """Subscribe to neighbour discovery events to get notified when new neighbour peers are discovered or lost

        Returns
        -------
        ManagedStream
        """
        neighbour_event_stream = self.stub.SubscribeEvents(self.empty)
        return ManagedStream(neighbour_event_stream)

    async def get_own_peer_object(self) -> script_pb2.Peer:
        """Get the Peer object of the current node

        Returns
        -------
        script_pb2.Peer
        """

        peer = await self.stub.GetOwnId(self.empty)
        return peer

    async def get_own_id(self) -> str:
        """Get the peer_id of the current node

        Returns
        -------
        peer_id : str
        """

        peer = await self.stub.GetOwnId(self.empty)
        return peer.peer_id
