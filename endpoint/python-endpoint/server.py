from fastapi import FastAPI, WebSocket, WebSocketDisconnect, HTTPException
from fastapi.responses import HTMLResponse
from pydantic import BaseModel
import asyncio
import argparse

app = FastAPI()

class Deployment(BaseModel):
    peer: str
    image: str

# Asynchronous data-generating functions
async def data_generator_1():
    while True:
        await asyncio.sleep(1)  # Simulate data generation delay
        yield "Data from generator 1"

async def data_generator_2():
    while True:
        await asyncio.sleep(2)  # Simulate data generation delay
        yield "Data from generator 2"

# Third asynchronous function to be triggered by PUT request
async def process_item(deployment):
    # Simulate some processing
    await asyncio.sleep(1)
    print(f"Deployed image {deployment.image} to peer {deployment.peer}")

# WebSocket endpoint 1
@app.websocket("/ws/peer-status")
async def websocket_endpoint_1(websocket: WebSocket):
    await websocket.accept()
    try:
        async for data in data_generator_1():
            await websocket.send_text(data)
    except WebSocketDisconnect:
        print("Client disconnected from generator 1")

# WebSocket endpoint 2
@app.websocket("/ws/mesh-status")
async def websocket_endpoint_2(websocket: WebSocket):
    await websocket.accept()
    try:
        async for data in data_generator_2():
            await websocket.send_text(data)
    except WebSocketDisconnect:
        print("Client disconnected from generator 2")

# PUT endpoint
@app.put("/docker-deploy/")
async def update_item(deployment: Deployment):
    await process_item(deployment)
    return {"status": "success", "name": deployment.peer, "image": deployment.image}

if __name__ == "__main__":
    import uvicorn

    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()

    uvicorn.run(app, host="0.0.0.0", port=args.port)
