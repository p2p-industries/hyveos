import asyncio
from asyncio.tasks import sleep
import os
from p2pindustries import P2PConnection


def print_response(peer_id: str, response):
    if response.WhichOneof('response') == 'data':
        print(f'Got response from {peer_id}: {response.data}')
    elif response.WhichOneof('response') == 'error':
        print(f'Got error from {peer_id}: {response.error}')


async def main():
    async with P2PConnection() as connection:
        discovery = connection.get_discovery_service()
        req_resp = connection.get_request_response_service()

        my_peer_id = await discovery.get_own_id()

        peer_id = os.environ['LISTENER_PEER_ID']

        response = await req_resp.send_request(peer_id, f'Hello from {my_peer_id}!')
        print_response(peer_id, response)

        await sleep(5)

        response = await req_resp.send_request(peer_id, b'Hello again!')
        print_response(peer_id, response)


if __name__ == '__main__':
    asyncio.run(main())
