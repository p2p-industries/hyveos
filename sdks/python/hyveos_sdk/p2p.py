import grpc
import os
from typing import Optional

from .services.db import DBService
from .services.debug import DebugService
from .services.dht import DHTService
from .services.discovery import DiscoveryService
from .services.file_transfer import FileTransferService
from .services.gossip_sub import GossipSubService
from .services.request_response import RequestResponseService

class Connection:
    """
    A connection to the HyveOS runtime.

    This class is used to establish a connection to the HyveOS runtime.
    It is used as a context manager to ensure that the connection is properly closed when it is no longer needed.

    By default, the connection to the HyveOS runtime will be made through the scripting bridge,
    i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable will be used to communicate with the runtime.

    If another connection type is desired, you can specify either the `socket_path` or `uri` argument when creating the connection.

    Example
    -------

    ```python
    from hyveos_sdk import Connection

    async def main():
        async with Connection() as conn:
            discovery = conn.get_discovery_service()
            peer_id = await discovery.get_own_id()

            print(f'My peer ID: {peer_id}')
    ```
    """

    def __init__(self, socket_path: Optional[str] = None, uri: Optional[str] = None):
        """
        Establishes a connection to the HyveOS runtime.

        By default, the connection to the HyveOS runtime will be made through the scripting bridge,
        i.e., the Unix domain socket specified by the `HYVEOS_BRIDGE_SOCKET` environment variable will be used to communicate with the runtime.

        If another connection type is desired, you can specify either the `socket_path` or `uri` parameter.

        Parameters
        ----------
        socket_path : str, optional
            A custom path to a Unix domain socket to connect to.
            The socket path should point to a Unix domain socket that the HyveOS runtime is listening on.

            Mutually exclusive with `uri`.
        uri : str, optional
            A URI to connect to over the network.
            The URI should be in the format `http://<host>:<port>`.
            A HyveOS runtime should be listening at the given address.

            Mutually exclusive with `socket_path`.

        Raises
        ------
        ValueError
            If both `socket_path` and `uri` are provided.
        """

        if socket_path is not None:
            if uri is not None:
                raise ValueError('Only one of `socket_path` and `uri` can be provided')
            self._conn = grpc.aio.insecure_channel(
                f'unix://{socket_path}',
                options=(('grpc.default_authority', 'localhost'),),
            )
        elif uri is not None:
            self._conn = grpc.aio.insecure_channel(uri)
        else:
            bridge_socket_path=os.environ['P2P_INDUSTRIES_BRIDGE_SOCKET']
            self._conn = grpc.aio.insecure_channel(
                f'unix://{bridge_socket_path}',
                options=(('grpc.default_authority', 'localhost'),),
            )

    async def __aenter__(self) -> 'OpenedConnection':
        return OpenedConnection(self)

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self._conn.close()


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
            discovery = conn.get_discovery_service()
            peer_id = await discovery.get_own_id()

            print(f'My peer ID: {peer_id}')
    ```
    """

    def __init__(self, conn: Connection):
        self._conn = conn._conn

    def get_db_service(self) -> DBService:
        """
        Returns a handle to the database service.

        Returns
        -------
        DBService
            A handle to the database service.
        """

        return DBService(self._conn)

    def get_debug_service(self) -> DebugService:
        """
        Returns a handle to the debug service.

        Returns
        -------
        DebugService
            A handle to the debug service.
        """

        return DebugService(self._conn)

    def get_dht_service(self) -> DHTService:
        """
        Returns a handle to the DHT service.

        Returns
        -------
        DHTService
            A handle to the DHT service.
        """

        return DHTService(self._conn)

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

        return FileTransferService(self._conn)

    def get_gossip_sub_service(self) -> GossipSubService:
        """
        Returns a handle to the gossipsub service.

        Returns
        -------
        GossipSubService
            A handle to the gossipsub service.
        """

        return GossipSubService(self._conn)

    def get_request_response_service(self) -> RequestResponseService:
        """
        Returns a handle to the request-response service.

        Returns
        -------
        RequestResponseService
            A handle to the request-response service.
        """

        return RequestResponseService(self._conn)
