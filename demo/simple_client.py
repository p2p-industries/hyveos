from gpiozero import Button, LED
import asyncio
import logging

import grpc
import script_pb2
import script_pb2_grpc

led = LED(17)
switch = Button(27)

async def send_request(stub, peer_id, data, seq):
    request = script_pb2.Request(peer_id=peer_id, data=data.encode(), seq=seq)
    response = await stub.Send(request)
    print(f"Send Response: {response.data.decode()} with peer {peer_id}")

async def handle_switch(stub, peer_id):
    await send_request(stub, peer_id, "INIT", 0)

    switch.when_pressed = lambda: asyncio.create_task(send_request(stub, peer_id, "ON", 0))
    switch.when_released = lambda: asyncio.create_task(send_request(stub, peer_id, "OFF", 0))

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

async def run() -> None:
    socket_path = "/var/run/p2p-bridge.sock"
    async with grpc.aio.insecure_channel(f'unix:{socket_path}') as channel:
        stubDiscovery = script_pb2_grpc.DiscoveryStub(channel)
        stubReqResp = script_pb2_grpc.ReqRespStub(channel)

        discovered_peer_id = await discover_peer(stubDiscovery)
        print(f"Discovered peer ID: {discovered_peer_id}")

        # Start handling switch events
        switch_task = asyncio.create_task(handle_switch(stubReqResp, discovered_peer_id))
        
        # Start receiving requests
        receive_task = asyncio.create_task(receive_requests(stubReqResp))

        await switch_task
        await receive_task

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(run())