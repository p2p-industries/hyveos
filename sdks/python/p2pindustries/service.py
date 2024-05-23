import asyncio
import grpc
import scripting.script_pb2 as script_pb2
import scripting.script_pb2_grpc as script_pb2_grpc
from dataclasses import dataclass


# ----- STRUCTS for API USERS which encapsulate data -----


# API USER work on a RECEIVED message
@dataclass
class Message:
    data: bytes
    hasTopic: bool
    topic: str


# API USER handling of topics for a script
@dataclass
class Topic:
    topic: str


# API USER handling of a received GossipSub Message
@dataclass
class GossipSubMessage:
    sender_peer_id: str
    message: Message


# API USER handling of fetched DHT Record
@dataclass
class DHTRecord:
    key: str
    hasValue: bool
    value: bytes


# API USER handling of fetched Peers that are providers of a DHTKey. A wrapper around string
@dataclass
class Peer:
    peer_id: str


# ----- HELPER FUNCTION -----


async def construct_message(self, data, topic=None):
    payload = data.encode()
    msg = script_pb2.Message(payload)
    if topic is not None:
        msg = script_pb2.Message(payload, topic)
    return msg


# ----- CLASSES for gRPC connection -----


class P2PConnection:
    def __init__(self, socket_path='/var/run/p2p-bridge.sock'):
        self.channel = grpc.aio.insecure_channel(
            f'unix:{socket_path}', options=(('grpc.default_authority', 'localhost'),)
        )


class RequestResponseService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubReqResp = script_pb2_grpc.ReqRespStub(self.channel)
        # maintaining a list of topics subscribed to?
        self.topics = []

    async def construct_response(self, data):
        return script_pb2.Response(data.encode())

    async def send_request(self, peer_id, data, topic):
        msg = construct_message(data, topic)
        send_request = script_pb2.SendRequest(peer_id=peer_id, msg=msg)
        response = await self.stubReqResp.Send(send_request)
        return response

    async def receive_generator(self):
        async for receive_request in self.stubReqResp.Recv(self.empty):
            receive_request_msg = receive_request.msg
            receive_request_msg_data = receive_request_msg.data.decode()
            if receive_request_msg.HasField('topic'):
                hasTopic = True
                receive_request_msg_topic = receive_request_msg.topic
            else:
                hasTopic = False
                receive_request_msg_topic = None
            yield Message(
                data=receive_request_msg_data,
                hasTopic=hasTopic,
                topic=receive_request_msg_topic,
            )

    async def respond(self, seq: int, response):
        send_response = script_pb2.SendResponse(seq=seq, response=response)
        await self.stubReqResp.Respond(send_response)

    async def subscribe(self, topic: str):
        if Topic(topic=topic) not in self.topics:
            topicProto = script_pb2.Topic(topic)
            await self.stubReqResp.Subscribe(topicProto)
            self.topics.append(Topic(topic=topic))
        else:
            print(f'Already subscribed to topic-str {topic}')

    async def unsubscribe(self, topic: str):
        topicProto = script_pb2.Topic(topic)
        await self.stubReqResp.Unsubscribe(topicProto)
        try:
            self.topics.remove(Topic(topic=topic))
        except ValueError:
            print(f'{topic} is not a topic-string in the subscribed list')


class DiscoveryService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubDiscovery = script_pb2_grpc.DiscoveryStub(self.channel)
        # Maintaining a list of peer_ids that we are currently connected to
        self.neighbours_peerIDs = []

        asyncio.create_task(self.maintain_current_neighbours())

    async def maintain_current_neighbours(self):
        self.discover()
        while True:
            self.subscribe_events()

    async def subscribe_events(self):
        try:
            async for neighbourEvent in self.stubDiscovery.SubscribeEvents(self.empty):
                self._handle_neighbour_event(neighbourEvent)
        except grpc.RpcError as e:
            print(f'RPC failed: {e}')

    def _handle_neighbour_event(self, event):
        field = event.WhichOneOf('event')
        if field == 'init':
            neighbours_peers = event.init.peers
            self.neighbours_peerIDs = [peer.peer_id for peer in neighbours_peers]
        elif field == 'discovered':
            self.neighbours_peerIDs.append(event.discovered.peer_id)
        elif field == 'lost':
            self.neighbours_peerIDs.remove(event.lost.peer_id)
        else:
            print('Nothing has been set in this neighbourEvent!')

    async def discover(self):
        async for peer in self.stubDiscovery.GetCurrentNeighbors(self.empty):
            discovered = peer.peer_id
            self.discovered_list.append(discovered)

    async def getOwnId(self):
        peer = self.stubDiscovery.GetOwnId(self.empty)
        return peer.peer_id


class GossipSubService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubGossipSub = script_pb2_grpc.GossipSubStub(self.channel)
        self.subscriptions = []

    async def subscribe_to_topic(self, topic: str):
        if Topic(topic=topic) not in self.subscriptions:
            topicProto = script_pb2.Topic(topic)
            await self.stubReqResp.Subscribe(topicProto)
            self.subscriptions.append(Topic(topic=topic))
        else:
            print(f'Already subscribed to topic-str {topic}')

    async def unsubscribe_from_topic(self, topic):
        topicProto = script_pb2.Topic(topic)
        await self.stubReqResp.Unsubscribe(topicProto)
        try:
            self.subscriptions.remove(Topic(topic=topic))
        except ValueError:
            print(f'{topic} is not a topic-string in the subscribed list')

    async def publish(self, msg):
        assert msg.topic is not None

        await self.stubGossipSub.Publish(msg)

    async def receive_message(self):
        async for receive_gossipSubMessage in self.stubGossipSub.Recv(self.empty):
            sender_peer_id = receive_gossipSubMessage.peer_id
            gossipSubMsg = receive_gossipSubMessage.msg
            assert gossipSubMsg.HasField('topic')

            data = gossipSubMsg.data.decode()
            topic = gossipSubMsg.topic

            yield GossipSubMessage(
                sender_peer_id=sender_peer_id,
                message=Message(data=data, hasTopic=True, topic=topic),
            )


class DHTService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubDHT = script_pb2_grpc.DHTStub(self.channel)

    async def _construct_DHT_key(self, key):
        dht_key = script_pb2.DHTKey(key)
        return dht_key

    async def _construct_DHT_record(self, key, value):
        dht_key = self._construct_DHT_key(key)
        dht_record = script_pb2.DHTRecord(dht_key)
        if value is not None:
            dht_record = script_pb2.DHTRecord(dht_key, value)
        return dht_record

    async def put_record(self, dht_record):
        await self.stubDHT.PutRecord(dht_record)

    async def get_record(self, dht_key):
        fetched_dht_record = await self.stubDHT.GetRecord(dht_key)
        fetched_record_key = fetched_dht_record.key.key
        if fetched_dht_record.HasField('value'):
            hasValue = True
            value = fetched_dht_record.value.decode()
        else:
            hasValue = False
            value = None
        fetched_record_struct = DHTRecord(fetched_record_key, hasValue, value)
        return fetched_record_struct

    async def provide(self, dht_key):
        await self.stubDHT.Provide(dht_key)

    async def get_providers_generator(self, dht_key):
        async for peer in self.stubDHT.GetProviders(dht_key):
            yield Peer(peer_id=peer.peer_id)


class FileTransferService:
    def __init__(self, channel):
        self.channel = channel
        self.empty = script_pb2.Empty()
        self.stubFileTransfer = script_pb2_grpc.FileTransferStub(self.channel)

    async def _construct_id(self, value):
        id = script_pb2.ID(value=value)
        return id

    async def _construct_file_path(self, path):
        file_path = script_pb2.File(path)
        return file_path

    async def _construct_file(self, file_path):
        path = self._construct_file_path(file_path)
        cid = self._construct_id(id)
        file = script_pb2.File(path, cid)
        return file

    async def publish_file(self, file):
        await self.stubFileTransfer.PublishFile(file)

    async def get_file(self, id):
        file_path = await self.stubFileTransfer.GetFile(id)
        return file_path
