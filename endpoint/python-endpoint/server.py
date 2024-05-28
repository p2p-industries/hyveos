from fastapi import FastAPI, WebSocket
import asyncio
import uvicorn
from p2pindustries import P2PConnection


class TopologyRelayManager:
    def __init__(self, debug_service, queue: asyncio.Queue):
        self.connections = []
        self.debug_service = debug_service
        self.queue = queue
        self.graph = set()

    async def add(self, conn: WebSocket):
        await conn.accept()
        self.connections.append(conn)

    async def remove(self, conn: WebSocket):
        self.connections.remove(conn)

    async def broadcast(self):
        while True:
            event = await self.queue.get()
            for conn in self.connections:
                await conn.send_text(f"Peer: {event.peer.peer_id} had event {event.event.WhichOneof('event')}")

    async def listen(self):
        async with self.debug_service.get_mesh_topology() as mesh_events:
            async for data in mesh_events:
                await self.queue.put(data)


app = FastAPI()


async def websocket_endpoint(websocket: WebSocket, manager: TopologyRelayManager):
    await manager.add(websocket)
    try:
        while True:
            await websocket.receive_text()
    except Exception as e:
        print(f"Connection closed: {e}")
    finally:
        await manager.remove(websocket)
        await websocket.close()


async def main():
    async with P2PConnection() as p2p_conn:
        queue = asyncio.Queue()

        manager = TopologyRelayManager(p2p_conn.get_debug_service(), queue)

        app.add_websocket_route("/topology", lambda websocket: websocket_endpoint(websocket, manager))

        config = uvicorn.Config(app, host="0.0.0.0", port=8000)
        server = uvicorn.Server(config)

        await asyncio.gather(
            manager.broadcast(),
            manager.listen(),
            server.serve()
        )

if __name__ == "__main__":
    asyncio.run(main())
