import grpc

from sdks.python.p2pindustries.service import FileTransferService
from sdks.python.p2pindustries.services.debug import DebugService
from services.dht import DHTService


class P2PConnection:
    def __init__(self, socket_path: str = '/var/run/p2p-bridge.sock'):
        self.conn = self.channel = grpc.aio.insecure_channel(
            f'unix:{socket_path}',
            options=(('protocol.default_authority', 'localhost'),),
        )

    def get_dht_service(self):
        return DHTService(self.conn)

    def get_file_transfer_service(self):
        return FileTransferService(self.conn)

    def get_debug_service(self):
        return DebugService(self.conn)
