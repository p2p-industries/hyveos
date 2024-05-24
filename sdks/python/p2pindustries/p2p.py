import grpc

from sdks.python.p2pindustries.services.debug import DebugService
from sdks.python.p2pindustries.services.dht import DHTService
from sdks.python.p2pindustries.services.discovery import DiscoveryService
from sdks.python.p2pindustries.services.file_transfer import FileTransferService
from sdks.python.p2pindustries.services.gossip_sub import GossipSubService
from sdks.python.p2pindustries.services.request_response import RequestResponseService


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

    def get_dht_service(self):
        return DHTService(self.conn)

    def get_discovery_service(self):
        return DiscoveryService(self.conn)

    def get_file_transfer_service(self):
        return FileTransferService(self.conn)

    def get_debug_service(self):
        return DebugService(self.conn)

    def get_gossip_sub_service(self):
        return GossipSubService(self.conn)

    def get_request_response_service(self):
        return RequestResponseService(self.conn)
