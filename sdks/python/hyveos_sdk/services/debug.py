from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import DebugStub
from ..protocol.bridge_pb2 import MeshTopologyEvent, MessageDebugEvent, Empty
from .stream import ManagedStream


class DebugService:
    """
    A handle to the debug service.

    Exposes methods to interact with the debug service,
    such as subscribing to mesh topology events and message debug events.
    """

    def __init__(self, conn: Channel):
        self.stub = DebugStub(conn)

    def subscribe_mesh_topology(self) -> ManagedStream[MeshTopologyEvent]:
        """
        Subscribes to mesh topology events.

        Returns a stream of mesh topology events. The stream will emit an event whenever the mesh
        topology changes.

        For each peer in the mesh, it is guaranteed that the stream will first emit an `init` event
        when it enters the mesh, followed only by `discovered` and `lost` events,
        until the peer leaves the mesh.

        Returns
        -------
        stream : ManagedStream[MeshTopologyEvent]
            A stream of mesh topology events
        """
        stream = self.stub.SubscribeMeshTopology(Empty())
        return ManagedStream(stream)

    def subscribe_messages(self) -> ManagedStream[MessageDebugEvent]:
        """
        Subscribes to message debug events.

        Returns a stream of mesh debug events. The stream will emit an event whenever a request,
        response, or gossipsub message is sent by a peer in the mesh.

        Returns
        -------
        stream : ManagedStream[MessageDebugEvent]
            A stream of message debug events
        """
        stream = self.stub.SubscribeMessages(Empty())
        return ManagedStream(stream)
