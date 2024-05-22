import asyncio
from websockets.server import serve
from p2pindustries import GossipSubService
import json


async def main():
    gossip = GossipSubService()

    async def handler(websocket):
        async for data in gossip.recv():
            await websocket.send(json.dumps(data))

    async with serve(handler, "localhost", 8765):
        await asyncio.Future()


asyncio.run(main())
