from ..protocol.script_pb2_grpc import DHTStub
from ..protocol.script_pb2 import DHTPutRecord, DHTKey, Topic, DHTGetRecord, Peer
from stream import ManagedStream
from util import enc


class DHTService:
    """
    Exposes the distributed hash table present in the p2p network
    """

    def __init__(self, conn):
        self.stub = DHTStub(conn)

    async def put_record(self, topic: str, key: str | bytes, value: str) -> None:
        """
        Puts a record in the DHT table under a specific topic
        """
        await self.stub.PutRecord(
            DHTPutRecord(key=DHTKey(topic=Topic(topic), key=enc(key)), value=enc(value))
        )

    async def get_record(self, topic: str, key: str | bytes) -> DHTGetRecord:
        """
        Retrieved a record in the DHT table under a specific topic
        """
        record = await self.stub.GetRecord(DHTKey(topic=Topic(topic), key=enc(key)))
        return record

    async def provide(self, topic: str, key: str | bytes) -> None:
        """
        Marks the peer as a provider of a record under a specific topic
        """
        await self.stub.Provide(DHTKey(topic=Topic(topic), key=enc(key)))

    async def get_providers(self, topic: str, key: str | bytes) -> ManagedStream[Peer]:
        """
        Returns an asynchronous iterator representing the providers of a record under a specific topic
        """
        stream = self.stub.GetProviders(DHTKey(topic=Topic(topic), key=enc(key)))
        return ManagedStream(stream)
