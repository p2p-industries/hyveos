import asyncio
import logging
from service import P2PConnection, RequestResponseService, DiscoveryService, GossipSubService, DHTService, FileTransferService


##########------ USER DEFINED SCRIPT ------##########
from gpiozero import Button, LED

led = LED(17)
switch = Button(27)

queue = asyncio.Queue()

def handle_switch(request_queue, loop):
    def put_request(data):
        loop.call_soon_threadsafe(request_queue.put_nowait, data)
    
    switch.when_pressed = lambda: put_request("ON")
    switch.when_released = lambda: put_request("OFF")

async def request_worker(queue, requestResponseService: RequestResponseService, peer_id, topic):
    while True:
        data = await queue.get()
        response = requestResponseService.send_request(peer_id, data, topic)
        queue.task_done()

async def receive_respond(requestResponseService: RequestResponseService):
    async for receive_message in requestResponseService.receive_generator():
        request_data = receive_message.data.decode()
        if request_data == "ON":
            led.on()
        elif request_data == "OFF":
            led.off()
        requestResponseService.respond(0, "OK")

async def init_neighbours(discovery_service: DiscoveryService):
    discovery_service.discover()
    return discovery_service.neighbours_peerIDs

# TODO: Let the API handle this automatically
async def maintain_current_neighbours(discovery_service: DiscoveryService):
    init_neighbours = init_neighbours(discovery_service)
    while True:
        discovery_service.subscribe_events()

# every script uses an async run() function
async def run() -> None:
    channel = P2PConnection()
    event_loop = asyncio.get_event_loop()

    requestResponseService = RequestResponseService(channel)
    discoveryService = DiscoveryService(channel)
    gossipSubService = GossipSubService(channel)
    dhtService = DHTService(channel)
    fileTransferService = FileTransferService(channel)
    

    handle_switch(queue, event_loop)


    receive_respond_task = asyncio.create_task(receive_respond(requestResponseService))

    # TODO: Figure this stuff out
    request_worker_task = []
    for i, neighbour_peer_id in enumerate(discoveryService.neighbours_peerIDs):
        for j, Topic in enumerate(gossipSubService.subscriptios):
            topic = Topic.topic
            request_worker_task.append(asyncio.create_task(request_worker(queue, requestResponseService, neighbour_peer_id, topic)))

    await receive_respond_task

    # TODO: Obviously this doesn't work
    for i, _ in enumerate(discoveryService.neighbours_peerIDs):
        for j, _ in enumerate(gossipSubService.subscriptions):
            await request_worker_task[i]
    

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run())
