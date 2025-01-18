import aiohttp
import itertools
import os
import shutil
import yarl

from grpc.aio import Channel
from ..protocol.bridge_pb2_grpc import FileTransferStub
from ..protocol.bridge_pb2 import FilePath, ID, CID
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
    A handle to the file transfer service.

    Exposes methods to interact with the file transfer service, like for publishing and getting files.
    """

    @abstractmethod
    async def publish(self, file_path: Path | str) -> FileToken:
        """
        Publishes a file in the mesh network and returns its content ID.

        Before it's published, the file is copied to the shared directory if it is not already
        there. By default, the shared directory is defined by the `HYVEOS_BRIDGE_SHARED_DIR`
        environment variable. However, it can be set to a custom path when initializing the
        connection to the hyveOS runtime.

        Parameters
        ----------
        file_path : Path | str
            The local path to the file to publish

        Returns
        -------
        file_token : FileToken
            The content ID of the published file
        """
        pass

    @abstractmethod
    async def get(self, file_token: FileToken) -> FilePath:
        """
        Retrieves a file from the mesh network and returns its path.

        When the local runtime doesn't own a copy of this file yet, it downloads it from one of its peers.
        Afterwards, or if it was already locally available, the file is copied
        into the shared directory, which is defined by the `HYVEOS_BRIDGE_SHARED_DIR` environment
        variable.

        Parameters
        ----------
        file_token : FileToken
            The content ID of the file to retrieve

        Returns
        -------
        file_path : FilePath
            The local path to the retrieved file
        """
        pass


class GrpcFileTransferService(FileTransferService):
    def __init__(self, conn: Channel, shared_dir_path: Optional[Path]):
        self.stub = FileTransferStub(conn)
        self.shared_dir_path = shared_dir_path

    async def publish(self, file_path: Path | str) -> FileToken:
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

        cid = await self.stub.Publish(FilePath(path=str(path)))
        return FileToken(cid.hash, cid.id.ulid)

    async def get(self, file_token: FileToken) -> FilePath:
        return await self.stub.Get(
            CID(hash=enc(file_token.hash), id=ID(ulid=file_token.id))
        )


class NetworkFileTransferService(FileTransferService):
    def __init__(self, uri: str, session: aiohttp.ClientSession):
        self.base_url = yarl.URL(uri)
        self.session = session

    async def publish(self, file_path: Path | str) -> FileToken:
        file_name = os.path.basename(file_path)
        url = self.base_url.joinpath('file-transfer/publish').joinpath(file_name)

        with open(file_path, 'rb') as f:
            async with self.session.post(url, data=f) as resp:
                data = await resp.json()
                return FileToken(data['hash'], data['id'])

    async def get(self, file_token: FileToken) -> FilePath:
        base_path = '/tmp'
        file_path = os.path.join(base_path, file_token.id)

        for i in itertools.count(start=1):
            if not os.path.exists(file_path):
                break

            file_path = os.path.join(base_path, f'{file_token.id}_{i}')

        url = self.base_url.joinpath('file-transfer/get')
        params = {'hash': file_token.hash, 'id': file_token.id}

        async with self.session.get(url, params=params) as resp:
            with open(file_path, 'wb') as f:
                async for chunk in resp.content.iter_chunked(1024):
                    f.write(chunk)

        return FilePath(path=file_path)
