from grpc.aio import Channel
from ..protocol.script_pb2_grpc import DebugStub
from ..protocol.script_pb2 import MeshTopologyEvent, Empty
from .stream import ManagedStream


class DebugService:
    """
    Exposes various debugging functionalities
    """

    def __init__(self, conn: Channel):
        self.stub = DebugStub(conn)

    def get_mesh_topology(self) -> ManagedStream[MeshTopologyEvent]:
        """
        Returns a stream of mesh topology events to observe the
        underlying connectivity state of the network
        """
        stream = self.stub.SubscribeMeshTopology(Empty())
        return ManagedStream(stream)
