import aiohttp
import grpc
import os
from pathlib import Path
from typing import Optional

from .protocol.bridge_pb2_grpc import DiscoveryStub
from .protocol.bridge_pb2 import Empty
from .services.apps import AppsService
from .services.debug import DebugService
from .services.discovery import DiscoveryService
from .services.file_transfer import (
    FileTransferService,
    GrpcFileTransferService,
    NetworkFileTransferService,
)
from .services.kv import KVService
from .services.local_kv import LocalKVService
from .services.neighbours import NeighboursService
from .services.pub_sub import PubSubService
from .services.req_res import RequestResponseService


class Connection:
    """
    A connection to the hyveOS runtime.

    This class is used to establish a connection to the HyveOS runtime.
    It is used as a context manager to ensure that the connection is properly closed when it is no longer needed.

    By default, the connection to the HyveOS runtime will be made through the scripting bridge,
    i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable will be used to communicate with the runtime.

    If another connection type is desired, you can specify either the `socket_path` and `shared_dir_path` parameters,
    or the `uri` parameter when creating the connection.

    Example
    -------

    ```python
    from hyveos_sdk import Connection

    async def main():
        async with Connection() as conn:
            peer_id = await conn.get_id()

            print(f'My peer ID: {peer_id}')
    ```
    """

    _conn: grpc.aio.Channel
    _shared_dir_path: Optional[Path]
    _uri: Optional[str]
    _session: Optional[aiohttp.ClientSession]

    def __init__(
        self,
        socket_path: Optional[Path | str] = None,
        shared_dir_path: Optional[Path | str] = None,
        uri: Optional[str] = None,
    ):
        """
        Establishes a connection to the HyveOS runtime.

        By default, the connection to the HyveOS runtime will be made through the scripting bridge,
        i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable will be used to communicate with the runtime.

        If another connection type is desired, you can specify either the `socket_path` and `shared_dir_path` parameters,
        or the `uri` parameter.

        Parameters
        ----------
        socket_path : Path | str, optional
            A custom path to a Unix domain socket to connect to.
            The socket path should point to a Unix domain socket that the HyveOS runtime is listening on.

            Mutually exclusive with `uri`. If `socket_path` is provided, `shared_dir_path` must also be provided.
        shared_dir_path : Path | str, optional
            A path to a directory where the runtime expects files provided with the file-transfer service to be stored.

            Mutually exclusive with `uri`. Must be provided if `socket_path` is provided.
        uri : str, optional
            A URI to connect to over the network.
            The URI should be in the format `http://<host>:<port>`.
            A HyveOS runtime should be listening at the given address.

            Mutually exclusive with `socket_path` and `shared_dir_path`.

        Raises
        ------
        ValueError
            If both `socket_path` and `uri` are provided.
        """

        shared_dir_path = (
            Path(shared_dir_path)
            if isinstance(shared_dir_path, str)
            else shared_dir_path
        )
        self._shared_dir_path = (
            shared_dir_path.resolve(strict=True)
            if shared_dir_path is not None
            else None
        )
        self._uri = uri
        self._session = None

        if socket_path is not None:
            if shared_dir_path is None:
                raise ValueError(
                    '`shared_dir_path` must be provided when `socket_path` is provided'
                )
            if uri is not None:
                raise ValueError(
                    'Only one of `socket_path` and `shared_dir_path`, or `uri` can be provided'
                )
            self._conn = grpc.aio.insecure_channel(
                f'unix://{socket_path}',
                options=(('grpc.default_authority', 'localhost'),),
            )
        elif uri is not None:
            if shared_dir_path is not None:
                raise ValueError(
                    '`shared_dir_path` cannot be provided when `uri` is provided'
                )
            self._conn = grpc.aio.insecure_channel(uri)
            self._session = aiohttp.ClientSession()
        elif shared_dir_path is not None:
            raise ValueError(
                '`shared_dir_path` cannot be provided without `socket_path`'
            )
        else:
            bridge_socket_path = os.environ['HYVEOS_BRIDGE_SOCKET']
            self._conn = grpc.aio.insecure_channel(
                f'unix://{bridge_socket_path}',
                options=(('grpc.default_authority', 'localhost'),),
            )

    async def __aenter__(self) -> 'OpenedConnection':
        return OpenedConnection(self)

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self._conn.close()

        if self._session is not None:
            await self._session.close()


class OpenedConnection:
    """
    An opened connection to the HyveOS runtime.

    This class provides access to the various services provided by HyveOS.

    An instance of this class is obtained by entering a `Connection` context manager.

    Example
    -------

    ```python
    from hyveos_sdk import Connection

    async def main():
        async with Connection() as conn:
            peer_id = await conn.get_id()

            print(f'My peer ID: {peer_id}')
    ```
    """

    _conn: grpc.aio.Channel
    _shared_dir_path: Optional[Path]
    _uri: Optional[str]
    _session: Optional[aiohttp.ClientSession]

    def __init__(self, conn: Connection):
        self._conn = conn._conn
        self._shared_dir_path = conn._shared_dir_path
        self._uri = conn._uri
        self._session = conn._session

    def get_apps_service(self) -> AppsService:
        """
        Returns a handle to the application management service.

        Returns
        -------
        AppsService
            A handle to the application management service.
        """
        return AppsService(self._conn)

    def get_debug_service(self) -> DebugService:
        """
        Returns a handle to the debug service.

        Returns
        -------
        DebugService
            A handle to the debug service.
        """
        return DebugService(self._conn)

    def get_discovery_service(self) -> DiscoveryService:
        """
        Returns a handle to the discovery service.

        Returns
        -------
        DiscoveryService
            A handle to the discovery service.
        """
        return DiscoveryService(self._conn)

    def get_file_transfer_service(self) -> FileTransferService:
        """
        Returns a handle to the file transfer service.

        Returns
        -------
        FileTransferService
            A handle to the file transfer service.
        """
        if self._uri is not None and self._session is not None:
            return NetworkFileTransferService(self._uri, self._session)
        else:
            return GrpcFileTransferService(self._conn, self._shared_dir_path)

    def get_kv_service(self) -> KVService:
        """
        Returns a handle to the distributed key-value store service.

        Returns
        -------
        KVService
            A handle to the distributed key-value store service.
        """
        return KVService(self._conn)

    def get_local_kv_service(self) -> LocalKVService:
        """
        Returns a handle to the local key-value store service.

        Returns
        -------
        LocalKVService
            A handle to the local key-value store service.
        """
        return LocalKVService(self._conn)

    def get_neighbours_service(self) -> NeighboursService:
        """
        Returns a handle to the neighbours service.

        Returns
        -------
        NeighboursService
            A handle to the neighbours service.
        """
        return NeighboursService(self._conn)

    def get_pub_sub_service(self) -> PubSubService:
        """
        Returns a handle to the pub-sub service.

        Returns
        -------
        PubSubService
            A handle to the pub-sub service.
        """
        return PubSubService(self._conn)

    def get_request_response_service(self) -> RequestResponseService:
        """
        Returns a handle to the request-response service.

        Returns
        -------
        RequestResponseService
            A handle to the request-response service.
        """
        return RequestResponseService(self._conn)

    async def get_id(self) -> str:
        """
        Returns the peer ID of the local runtime.

        Returns
        -------
        str
            The peer ID of the runtime.
        """
        peer = await DiscoveryStub(self._conn).GetOwnId(Empty())
        return peer.peer_id
