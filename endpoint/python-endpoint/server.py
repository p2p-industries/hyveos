import os
import json
import asyncio
from typing import List

import uvicorn
from fastapi import FastAPI, WebSocket
from p2pindustries import P2PConnection

class Event:
    source: str
    targets: List[str]
    event_type: str

    def __init__(self, source: str, targets: List[str], event_type: str):
        self.source = source
        self.targets = targets
        self.event_type = event_type

    def to_dict(self):
        return {'source': self.source.peer_id,
                'targets': self.targets,
                'event_type': self.event_type}

    def to_json(self):
        return json.dumps(self.to_dict())

    @staticmethod
    def parse_from_mesh_event(mesh_event):
        event = mesh_event.event
        event_type = event.WhichOneof('event')

        if event_type == 'init':
            targets = [peer.peer_id for peer in event.init.peers]
        elif event_type == 'discovered':
            targets = [event.discovered.peer_id]
        elif event_type == 'lost':
            targets = [event.lost.peer_id]
        else:
            targets = []

        return Event(source=mesh_event.peer,
                     targets=targets,
                     event_type=event_type)


class TopologyRelayManager:
    def __init__(self, debug_service, queue: asyncio.Queue):
        self.connections = []
        self.debug_service = debug_service
        self.queue = queue
        self.events = set()

    async def add(self, conn: WebSocket):
        await conn.accept()
        await conn.send_text(json.dumps([event.to_dict() for event in self.events]))

        self.connections.append(conn)

    async def remove(self, conn: WebSocket):
        self.connections.remove(conn)

    async def broadcast(self):
        while True:
            event = await self.queue.get()
            for conn in self.connections:
                await conn.send_text(event.to_json())

    async def listen(self):
        async with self.debug_service.get_mesh_topology() as mesh_events:
            async for data in mesh_events:
                event = Event.parse_from_mesh_event(data)
                print(f"Detected change in Topology: {event.to_json()}")
                self.events.add(event)
                await self.queue.put(event)


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
    async with P2PConnection() as conn:
        queue = asyncio.Queue()

        manager = TopologyRelayManager(conn.get_debug_service(), queue)

        app.add_websocket_route("/topology", lambda websocket: websocket_endpoint(websocket, manager))

        config = uvicorn.Config(app, host="0.0.0.0", port=int(os.environ['PYTHON_ENDPOINT_PORT']))
        server = uvicorn.Server(config)

        await asyncio.gather(
            manager.broadcast(),
            manager.listen(),
            server.serve()
        )

if __name__ == "__main__":
    asyncio.run(main())
