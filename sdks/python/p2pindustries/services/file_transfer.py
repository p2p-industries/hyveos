from ..protocol.script_pb2_grpc import FileTransferStub
from ..protocol.script_pb2 import File, FilePath, ID, CID
from util import enc
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

    def __init__(self, conn):
        self.stub = FileTransferStub(conn)

    async def publish_file(self, file_path: str, file_id: str) -> FileToken:
        """
        Publishes a file to the p2p network
        :param file_path: The local path to the file
        :param file_id: An ID for the file
        :return: A CID token that can be used by other peers to download the file
        """
        cid = await self.stub.PublishFile(
            file=File(FilePath(file_path), id=ID(file_id))
        )
        return FileToken(cid.hash, cid.id)

    async def get_file(self, file_token: FileToken) -> FilePath:
        """
        Downloads a file from the p2p network
        :param file_token: The has
        :return:
        """
        path = await self.stub.GetFile(CID(enc(file_token.hash), ID(file_token.id)))
        return path
