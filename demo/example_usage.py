import asyncio
import logging
from p2p_sdk import P2PConnection, RequestResponseService, DiscoveryService
from typing import Optional


async def get_first_neighbour(service: DiscoveryService) -> Optional[str]:
    async with service.discovery_events() as events:
        async for event in events:
            if event.is_init():
                return event.init().peer_id
    return None


async def handle_requests(
    service: RequestResponseService, topic: Optional[str] = None, regex: bool = False
):
    async with service.receive_requests(topic, regex) as requests:
        async for request in requests:
            print(request)
            await request.send_response(b'Hello from the other side!')


async def main():
    async with P2PConnection() as connection:
        discovery = connection.discovery_service()
        req_resp = connection.request_response_service()

        peer_id = await get_first_neighbour(discovery)

        if peer_id is None:
            logging.error('No peers found')
            return

        logging.info(f'Found peer: {peer_id}')

        asyncio.create_task(handle_requests(req_resp))
        asyncio.create_task(handle_requests(req_resp, topic='foo'))
        asyncio.create_task(handle_requests(req_resp, topic=r'bar[0-9]+', regex=True))

        response = await req_resp.send_request(peer_id, b'Hello!', topic='topic1')
        del response


if __name__ == '__main__':
    asyncio.run(main())
