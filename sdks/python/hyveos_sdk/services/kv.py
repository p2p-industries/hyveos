from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import KVStub
from ..protocol.bridge_pb2 import DHTRecord, DHTKey, Data, Topic
from .util import enc


class KVService:
    """
    A handle to the distributed key-value store service.

    Exposes methods to interact with the key-value store service,
    like for putting records into the key-value store or getting records from it.
    """

    def __init__(self, conn: Channel):
        self.stub = KVStub(conn)

    async def put_record(self, topic: str, key: str | bytes, value: str) -> None:
        """
        Puts a record into the key-value store.

        Parameters
        ----------
        topic : str
            The topic of the record
        key : str | bytes
            The key of the record
        value : str
            The value of the record
        """
        await self.stub.PutRecord(
            DHTRecord(
                key=DHTKey(topic=Topic(topic=topic), key=enc(key)),
                value=Data(data=enc(value)),
            )
        )

    async def get_record(self, topic: str, key: str | bytes) -> bytes | None:
        """
        Gets a record from the key-value store.

        Parameters
        ----------
        topic : str
            The topic of the record to retrieve
        key : str | bytes
            The key of the record to retrieve

        Returns
        -------
        value : bytes | None
            The value of the record or `None` if the record is not found
        """
        record = await self.stub.GetRecord(
            DHTKey(topic=Topic(topic=topic), key=enc(key))
        )
        if record.data is not None:
            return record.data.data
        else:
            return None

    async def remove_record(self, topic: str, key: str | bytes) -> None:
        """
        Removes a record from the key-value store.

        This only applies to the local node and only affects the network once the record expires.
        """
        await self.stub.RemoveRecord(DHTKey(topic=Topic(topic=topic), key=enc(key)))
