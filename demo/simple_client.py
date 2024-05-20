from gpiozero import Button, LED
import asyncio
import logging

import grpc
import script_pb2
import script_pb2_grpc

led = LED(17)
switch = Button(27)

def handle_switch(request_queue, loop):
    def put_request(data):
        loop.call_soon_threadsafe(request_queue.put_nowait, data)

    switch.when_pressed = lambda: put_request("ON")
    switch.when_released = lambda: put_request("OFF")

async def send_request(stub, peer_id, data, seq):
    print(f"Send Request: {data} with peer {peer_id}")
    request = script_pb2.Request(peer_id=peer_id, data=data.encode(), seq=seq)
    response = await stub.Send(request)
    print(f"Send Response: {response.data.decode()} with peer {peer_id}")

async def receive_requests(stub):
    empty = script_pb2.Empty()
    async for request in stub.Recv(empty):
        request_data = request.data.decode()
        if request_data == 'ON':
            led.on()
            print("Receive ON completed")
        elif request_data == 'OFF':
            led.off()
            print("Receive OFF completed")

        # Respond to the request
        response = script_pb2.Response(data=b'Received')
        send_response = script_pb2.SendResponse(seq=request.seq, response=response)
        await stub.Respond(send_response)
        print("Response completed")

async def discover_peer(stubDiscovery):
    empty = script_pb2.Empty()
    async for discovered in stubDiscovery.Discover(empty):
        return discovered.peer_id

async def request_worker(queue, stub, peer_id):
    while True:
        data = await queue.get()

        await send_request(stub, peer_id, data, 0)
        queue.task_done()

async def run() -> None:
    request_queue = asyncio.Queue()

    socket_path = "/home/pi/p2p-bridge.sock"
    async with grpc.aio.insecure_channel(f'unix:{socket_path}', options=(('grpc.default_authority', 'localhost'),)) as channel:
        stubDiscovery = script_pb2_grpc.DiscoveryStub(channel)
        stubReqResp = script_pb2_grpc.ReqRespStub(channel)

        discovered_peer_id = await discover_peer(stubDiscovery)
        print(f"Discovered peer ID: {discovered_peer_id}")

        # Start handling switch events
        loop = asyncio.get_event_loop()
        handle_switch(request_queue, loop)

        # Start receiving requests
        receive_task = asyncio.create_task(receive_requests(stubReqResp))

        # Start request worker
        request_worker_task = asyncio.create_task(request_worker(request_queue, stubReqResp, discovered_peer_id))

        await receive_task
        await request_worker_task

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run())
