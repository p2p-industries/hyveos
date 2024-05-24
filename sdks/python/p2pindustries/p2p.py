import grpc

from .services.debug import DebugService
from .services.dht import DHTService
from .services.discovery import DiscoveryService
from .services.file_transfer import FileTransferService
from .services.gossip_sub import GossipSubService
from .services.request_response import RequestResponseService


class P2PConnection:
    def __init__(self, socket_path: str = '/var/run/p2p-bridge.sock'):
        self.conn = self.channel = grpc.aio.insecure_channel(
            f'unix:{socket_path}',
            options=(('protocol.default_authority', 'localhost'),),
        )

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.conn.close()

    def get_dht_service(self) -> DHTService:
        return DHTService(self.conn)

    def get_discovery_service(self) -> DiscoveryService:
        return DiscoveryService(self.conn)

    def get_file_transfer_service(self) -> FileTransferService:
        return FileTransferService(self.conn)

    def get_debug_service(self) -> DebugService:
        return DebugService(self.conn)

    def get_gossip_sub_service(self) -> GossipSubService:
        return GossipSubService(self.conn)

    def get_request_response_service(self) -> RequestResponseService:
        return RequestResponseService(self.conn)
