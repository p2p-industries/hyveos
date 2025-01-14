from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import DiscoveryStub
from ..protocol.bridge_pb2 import DHTKey, Topic, Peer
from .stream import ManagedStream
from .util import enc


class DiscoveryService:
    """
    A handle to the discovery service.

    Exposes methods to interact with the discovery service,
    like for marking the local runtime as a provider for a discovery key
    or getting the providers for a discovery key.
    """

    def __init__(self, conn: Channel):
        self.stub = DiscoveryStub(conn)

    async def provide(self, topic: str, key: str | bytes) -> None:
        """
        Marks the local runtime as a provider for a discovery key.
        """
        await self.stub.Provide(DHTKey(topic=Topic(topic=topic), key=enc(key)))

    def get_providers(self, topic: str, key: str | bytes) -> ManagedStream[Peer]:
        """
        Gets the providers for a discovery key.

        Returns
        -------
        stream : ManagedStream[Peer]
            A stream of providers for the discovery key.
        """
        stream = self.stub.GetProviders(DHTKey(topic=Topic(topic=topic), key=enc(key)))
        return ManagedStream(stream)

    async def stop_providing(self, topic: str, key: str | bytes) -> None:
        """
        Stops providing a discovery key.
        """
        await self.stub.StopProviding(DHTKey(topic=Topic(topic=topic), key=enc(key)))
