from grpc.aio import Channel

from ..protocol.script_pb2_grpc import DBStub
from ..protocol.script_pb2 import DBRecord, DBKey, Data, OptionalData
from .util import enc


class DBService:
    """
    Exposes a key-value store to persistently store and retrieve data
    """

    def __init__(self, conn: Channel):
        self.stub = DBStub(conn)

    async def put(self, key: str, value: str | bytes) -> OptionalData:
        """
        Put a record into the key-value store and get the previous value if it exists
        """
        data = await self.stub.Put(DBRecord(key=key, value=Data(data=enc(value))))

        return data

    async def get(self, key: str) -> OptionalData:
        """
        Get a record from the key-value store
        """
        return await self.stub.Get(DBKey(key=key))
