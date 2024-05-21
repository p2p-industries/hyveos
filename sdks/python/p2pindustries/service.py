import grpc
import script_pb2
import script_pb2_grpc


class P2PConnection:
    def __init__(self, socket_path="/var/run/p2p-bridge.sock"):
        self.channel = grpc.aio.insecure_channel(f'unix:{socket_path}')


class RequestResponseService:
    def __init__(self, conn):
        self.conn = conn
        self.stub = script_pb2_grpc.ReqRespStub(conn.channel)

    async def request(self, peer_id, data, seq=0):
        request = script_pb2.Request(
            peer_id=peer_id,
            data=data.encode(),
            seq=seq)

        resp = await self.stub.Send(request)

        return resp

    async def get_requests(self):
        pass

    async def respond(self):
        pass


class DiscoveryService:
    def __init__(self, conn):
        self.conn = conn

    def discover(self):
        pass


def main():
    conn = P2PConnection()

    discovery_service = DiscoveryService(conn)
    reqres_service = RequestResponseService(conn)

    reqres_service.request()

    reqres_service.get()
    reqres_service.respond()
