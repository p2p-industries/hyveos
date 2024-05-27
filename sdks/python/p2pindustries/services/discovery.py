from ..protocol.script_pb2_grpc import DiscoveryStub
from ..protocol.script_pb2 import Peer, Empty, NeighbourEvent
from .stream import ManagedStream


class DiscoveryService:
    """
    Keeping track of neighbours
    """

    def __init__(self, conn):
        self.stub = DiscoveryStub(conn)
        self.empty = Empty()

    def discovery_events(self) -> ManagedStream[NeighbourEvent]:
        """
        Subscribe to neighbour discovery events to get notified when new neighbour peers are discovered or lost

        Returns
        -------
        stream : ManagedStream[NeighbourEvent]
            Iterator to handle the stream of neighbour events
        """
        neighbour_event_stream = self.stub.SubscribeEvents(self.empty)
        return ManagedStream(neighbour_event_stream)

    async def get_own_peer_object(self) -> Peer:
        """
        Get the Peer object of the current node

        Returns
        -------
        peer : Peer
            Own Peer object
        """

        peer = await self.stub.GetOwnId(self.empty)
        return peer

    async def get_own_id(self) -> str:
        """
        Get the peer_id of the current node

        Returns
        -------
        peer_id : str
            Own peer_id
        """

        peer = await self.stub.GetOwnId(self.empty)
        return peer.peer_id
