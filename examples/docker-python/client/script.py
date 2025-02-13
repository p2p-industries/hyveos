import asyncio
from asyncio.tasks import sleep
from typing import Optional
from hyveos_sdk import Connection, DiscoveryService


async def get_listener(discovery: DiscoveryService) -> Optional[str]:
    async with discovery.get_providers('identification', 'simple-listener') as providers:
        async for provider in providers:
            return provider.peer_id


def print_response(peer_id: str, response):
    if response.WhichOneof('response') == 'data':
        print(f'Got response from {peer_id}: {response.data}')
    elif response.WhichOneof('response') == 'error':
        print(f'Got error from {peer_id}: {response.error}')


async def main():
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        req_resp = connection.get_request_response_service()

        my_peer_id = await connection.get_id()

        peer_id = await get_listener(discovery)

        if peer_id is None:
            print('No listener found')
            return

        print(f'Found listener: {peer_id}')

        response = await req_resp.send_request(peer_id, f'Hello from {my_peer_id}!', 'foo')
        print_response(peer_id, response)

        await sleep(5)

        response = await req_resp.send_request(peer_id, b'Hello again!', 'foo')
        print_response(peer_id, response)


if __name__ == '__main__':
    asyncio.run(main())
