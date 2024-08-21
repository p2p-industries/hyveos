import asyncio
from p2pindustries import P2PConnection, RequestResponseService
from typing import Optional


async def handle_requests(
    service: RequestResponseService, topic: Optional[str] = None, regex: bool = False
):
    print('Waiting for requests...')
    async with service.receive(topic, regex) as requests:
        async for request in requests:
            print(f'Request from {request.peer_id}: {request.msg.data}')
            await service.respond(request.seq, b'Hello from the other side!')


async def main():
    async with P2PConnection() as connection:
        dht = connection.get_dht_service()
        req_resp = connection.get_request_response_service()

        await dht.provide('identification', 'simple-listener')

        await handle_requests(req_resp, 'foo')


if __name__ == '__main__':
    asyncio.run(main())
