import asyncio
import grpc
import scripting.script_pb2 as script_pb2
import scripting.script_pb2_grpc as script_pb2_grpc


class P2PConnection:
    def __init__(self, socket_path="/var/run/p2p-bridge.sock"):
        self.channel = grpc.aio.insecure_channel(f'unix:{socket_path}', options=(('grpc.default_authority', 'localhost'),))


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
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubDiscovery = script_pb2_grpc.DiscoveryStub(channel)
        # maintaining a list of peer_id's that we are currently connected to
        self.neighbours_list = []

    async def SubscribeEvents(self):
        async for event in self.stubDiscovery.SubscribeEvents(self.empty):


    async def discover(self):
        async for peer in self.stubDiscovery.GetCurrentNeighbors(self.empty):
           discovered = peer.peer_id
           self.discovered_list.append(discovered)

    async def Sub

class GossipSubService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()

    async def subscribe_to_topic(self, string):

    async def unsubscribe_from_topic(self, string):


    async def publish 

    async def recv(empty):
        

def main():
    channel = P2PConnection()

    discovery_service = DiscoveryService(channel)
    reqres_service = RequestResponseService(conn)

    reqres_service.request()

    reqres_service.get()
    reqres_service.respond()
