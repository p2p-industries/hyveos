from grpc.aio import Channel
from ..protocol.script_pb2_grpc import FileTransferStub
from ..protocol.script_pb2 import FilePath, ID, CID
from .util import enc

from dataclasses import dataclass


@dataclass
class FileToken:
    hash: str
    id: str

    def __str__(self):
        return self.hash + self.id


class FileTransferService:
    """
    Exposes a file transfer service that can be used to upload and download files
    in the p2p network
    """

    def __init__(self, conn: Channel):
        self.stub = FileTransferStub(conn)

    async def publish_file(self, file_path: str) -> FileToken:
        """
        Publishes a file to the p2p network
        :param file_path: The local path to the file
        :return: A CID token that can be used by other peers to download the file
        """
        # TODO: Add support for transferring files over the network
        cid = await self.stub.PublishFile(FilePath(path=file_path))
        return FileToken(cid.hash, cid.id.ulid)

    async def get_file(self, file_token: FileToken) -> FilePath:
        """
        Downloads a file from the p2p network
        :param file_token: The has
        :return:
        """
        # TODO: Add support for transferring files over the network
        return await self.stub.GetFile(
            CID(hash=enc(file_token.hash), id=ID(ulid=file_token.id))
        )
