import asyncio
from hyveos_sdk import Connection, RequestResponseService
from typing import Optional


async def handle_requests(
    service: RequestResponseService, topic: Optional[str] = None, regex: bool = False
):
    print('Waiting for requests...')
    async with service.receive(topic, regex) as requests:
        async for request in requests:
            print(f'Request from {request.peer.peer_id}: {request.msg.data}')
            await service.respond(request.seq, b'Hello from the other side!')


async def main():
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        req_resp = connection.get_request_response_service()

        await discovery.provide('identification', 'simple-listener')

        await handle_requests(req_resp, 'foo')


if __name__ == '__main__':
    asyncio.run(main())
