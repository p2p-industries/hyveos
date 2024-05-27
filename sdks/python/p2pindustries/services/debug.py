from ..protocol.script_pb2_grpc import DebugStub
from ..protocol.script_pb2 import MeshTopologyEvent
from .stream import ManagedStream


class DebugService:
    """
    Exposes various debugging functionalities
    """

    def __init__(self, conn):
        self.stub = DebugStub(conn)

    def get_mesh_topology(self) -> ManagedStream[MeshTopologyEvent]:
        """
        Returns a stream of mesh topology events to observe the
        underlying connectivity state of the network
        """
        stream = self.stub.SubscribeMeshTopology()
        return ManagedStream(stream)
