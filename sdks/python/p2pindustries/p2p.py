import grpc
import os

from .services.db import DBService
from .services.debug import DebugService
from .services.dht import DHTService
from .services.discovery import DiscoveryService
from .services.file_transfer import FileTransferService
from .services.gossip_sub import GossipSubService
from .services.request_response import RequestResponseService


class P2PConnection:
    def __init__(self, socket_path: str = os.environ['P2P_INDUSTRIES_BRIDGE_SOCKET']):
        self._conn = grpc.aio.insecure_channel(
            f'unix://{socket_path}',
            options=(('grpc.default_authority', 'localhost'),),
        )

    async def __aenter__(self) -> 'OpenedP2PConnection':
        return OpenedP2PConnection(self)

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self._conn.close()


class OpenedP2PConnection:
    def __init__(self, conn: P2PConnection):
        self._conn = conn._conn

    def get_db_service(self) -> DBService:
        return DBService(self._conn)

    def get_debug_service(self) -> DebugService:
        return DebugService(self._conn)

    def get_dht_service(self) -> DHTService:
        return DHTService(self._conn)

    def get_discovery_service(self) -> DiscoveryService:
        return DiscoveryService(self._conn)

    def get_file_transfer_service(self) -> FileTransferService:
        return FileTransferService(self._conn)

    def get_gossip_sub_service(self) -> GossipSubService:
        return GossipSubService(self._conn)

    def get_request_response_service(self) -> RequestResponseService:
        return RequestResponseService(self._conn)
