from grpc.aio import Channel

from ..protocol.bridge_pb2_grpc import LocalKVStub
from ..protocol.bridge_pb2 import LocalKVRecord, LocalKVKey, Data
from .util import enc


class LocalKVService:
    """
    A handle to the local key-value store service.

    Exposes methods to interact with the key-value store service,
    like putting and getting key-value records.
    The key-value store is local to the runtime and is not shared with other runtimes.
    However, it is persisted across restarts of the runtime.
    """

    def __init__(self, conn: Channel):
        self.stub = LocalKVStub(conn)

    async def put(self, key: str, value: str | bytes) -> bytes | None:
        """
        Puts a record into the key-value store.

        Returns the previous value if it exists, otherwise `None`.
        This only has local effects and does not affect other runtimes.
        However, the record is persisted across restarts of the runtime.

        Parameters
        ----------
        key : str
            The key of the record
        value : str | bytes
            The value of the record

        Returns
        -------
        value : bytes | None
            The previous value of the record or `None` if it did not exist
        """
        record = await self.stub.Put(LocalKVRecord(key=key, value=Data(data=enc(value))))
        if record.data is not None:
            return record.data.data
        else:
            return None

    async def get(self, key: str) -> bytes | None:
        """
        Gets a record from the key-value store if it exists.

        This will not return values from other runtimes.

        Parameters
        ----------
        key : str
            The key of the record to retrieve

        Returns
        -------
        value : bytes | None
            The value of the record or `None` if the record is not found
        """
        record = await self.stub.Get(LocalKVKey(key=key))
        if record.data is not None:
            return record.data.data
        else:
            return None
