from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import NeighboursStub
from ..protocol.bridge_pb2 import Empty, NeighbourEvent
from .stream import ManagedStream


class NeighboursService:
    """
    A handle to the neighbours service.

    Exposes methods to interact with the neighbours service, such as subscribing to neighbour events
    and getting the current set of neighbours.
    """

    def __init__(self, conn: Channel):
        self.stub = NeighboursStub(conn)
        self.empty = Empty()

    def subscribe(self) -> ManagedStream[NeighbourEvent]:
        """
        Subscribes to neighbour events.

        Returns a stream of neighbour events. The stream will emit an event whenever the local
        runtime detects a change in the set of neighbours. The stream is guaranteed to emit an
        `init` event directly after subscribing and only `discovered` and `lost` events afterwards.

        Returns
        -------
        stream : ManagedStream[NeighbourEvent]
            A stream of neighbour events
        """
        neighbour_event_stream = self.stub.Subscribe(self.empty)
        return ManagedStream(neighbour_event_stream)

    async def get(self) -> list[str]:
        """
        Returns the peer IDs of the current neighbours.

        Returns
        -------
        peers : list[str]
            A list of peer IDs of the current neighbours
        """

        peers = await self.stub.Get(self.empty)
        return [peer.peer_id for peer in peers.peers]
