from gpiozero import Button, LED
import asyncio
import logging

import grpc
import script_pb2
import script_pb2_grpc

from simple import switch_on, switch_off

led = LED(17)
switch = Button(27)

async def send_request(stub, peer_id, data, seq):
    request = script_pb2.Request(peer_id=peer_id, data=data, seq=seq)
    response = await stub.Send(request)
    print(f"Send Response: {response.data} with peer {peer_id}")

async def run() -> None:
    async with grpc.aio.insecure_channel("localhost:50051") as channel:
        stubDiscovery = script_pb2_grpc.DiscoveryStub(channel)
        stubReqResp = script_pb2_grpc.ReqRespStub(channel)

        empty = script_pb2.Empty()

        discovered_peer_id = ""

        async for discovered in stubDiscovery.Discover(empty):
            discovered_peer_id = discovered.peer_id
            break;

        async for request in stubReqResp.Recv(empty):
            request_data = request.data
            request_seq = request.seq
            if request_data == 'ON':
                led.on()
                print("Receive completed")
            response = script_pb2.Response(data='Turned ON')
            send_response = script_pb2.SendResponse(seq=request_seq, response=response)
            empty = await stubReqResp.Respond(send_response)
            print("Response completed")

        # Send a request. peer_id is the id of the peer I am sending a request to
        await send_request(stubReqResp, peer_id=discovered_peer_id, data="ON", seq=1)


if __name__ == "__main__":
    logging.basicConfig()
    asyncio.run(run())