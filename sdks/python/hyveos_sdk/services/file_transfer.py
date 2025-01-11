import aiohttp
import itertools
import os
import shutil
import yarl

from grpc.aio import Channel
from ..protocol.script_pb2_grpc import FileTransferStub
from ..protocol.script_pb2 import FilePath, ID, CID
from .util import enc

from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class FileToken:
    hash: str
    id: str

    def __str__(self):
        return self.hash + self.id


class FileTransferService(ABC):
    """
    Exposes a file transfer service that can be used to upload and download files
    in the p2p network
    """

    @abstractmethod
    async def publish_file(self, file_path: Path | str) -> FileToken:
        """
        Publishes a file to the p2p network

        Before it's published, the file is copied to the shared directory, if it's not already there.

        :param file_path: The local path to the file
        :return: A CID token that can be used by other peers to download the file
        """
        pass

    @abstractmethod
    async def get_file(self, file_token: FileToken) -> FilePath:
        """
        Downloads a file from the p2p network
        :param file_token: The has
        :return:
        """
        pass


class GrpcFileTransferService(FileTransferService):
    def __init__(self, conn: Channel, shared_dir_path: Optional[Path]):
        self.stub = FileTransferStub(conn)
        self.shared_dir_path = shared_dir_path

    async def publish_file(self, file_path: Path | str) -> FileToken:
        shared_dir = self.shared_dir_path or Path(
            os.environ['HYVEOS_BRIDGE_SHARED_DIR']
        )

        file_path = Path(file_path) if isinstance(file_path, str) else file_path
        file_path = file_path.resolve(strict=True)

        if shared_dir in file_path.parents:
            path = file_path
        else:
            path = shared_dir / file_path.name
            shutil.copy(file_path, path)

        cid = await self.stub.PublishFile(FilePath(path=str(path)))
        return FileToken(cid.hash, cid.id.ulid)

    async def get_file(self, file_token: FileToken) -> FilePath:
        return await self.stub.GetFile(
            CID(hash=enc(file_token.hash), id=ID(ulid=file_token.id))
        )


class NetworkFileTransferService(FileTransferService):
    def __init__(self, uri: str, session: aiohttp.ClientSession):
        self.base_url = yarl.URL(uri)
        self.session = session

    async def publish_file(self, file_path: Path | str) -> FileToken:
        file_name = os.path.basename(file_path)
        url = self.base_url.joinpath('file-transfer/publish-file').joinpath(file_name)

        with open(file_path, 'rb') as f:
            async with self.session.post(url, data=f) as resp:
                data = await resp.json()
                return FileToken(data['hash'], data['id'])

    async def get_file(self, file_token: FileToken) -> FilePath:
        base_path = '/tmp'
        file_path = os.path.join(base_path, file_token.id)

        for i in itertools.count(start=1):
            if not os.path.exists(file_path):
                break

            file_path = os.path.join(base_path, f'{file_token.id}_{i}')

        url = self.base_url.joinpath('file-transfer/get-file')
        params = {'hash': file_token.hash, 'id': file_token.id}

        async with self.session.get(url, params=params) as resp:
            with open(file_path, 'wb') as f:
                async for chunk in resp.content.iter_chunked(1024):
                    f.write(chunk)

        return FilePath(path=file_path)
