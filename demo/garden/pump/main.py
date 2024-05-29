import asyncio
from asyncio import Queue
from p2pindustries import DHTService, GossipSubService, P2PConnection, RequestResponseService
from json import dumps, loads
from gpiozero import Button, PWMOutputDevice
from datetime import datetime, timedelta
from time import time
from random import randint

FLOW_METER_PIN = 22

OUR_TIME_DELTA = timedelta(hours=1).total_seconds()


class Pump:
    device: PWMOutputDevice
    running: bool = False
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def __init__(self) -> None:
        self.device = PWMOutputDevice(12)

    async def __turn_up__(self):
        self.running = True
        start = time()
        while time() - start < 1:
            since_start = time() - start
            self.device.value = since_start
            await asyncio.sleep(0.01)
        self.device.on()

    async def __turn_down__(self):
        start = time()
        while time() - start < 1:
            since_start = time() - start
            self.device.value = 1 - since_start
            await asyncio.sleep(0.01)
        self.device.off()
        self.running = False

    async def start(self):
        if not self.running:
            await self.__turn_up__()

    async def stop(self):
        if self.running:
            await self.__turn_down__()

    def reduce(self):
        if self.running:
            self.device.value = 0.25

    async def __aenter__(self):
        await self.semaphore.acquire()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.stop()
        self.semaphore.release()




async def report_global_counter(conn: P2PConnection):
    gos = conn.get_gossip_sub_service()
    queue = Queue()

    flow_meter = Button(FLOW_METER_PIN)

    def on_click():
        queue.put_nowait(1)

    flow_meter.when_activated = on_click

    flow_count = 0

    while True:
        _ = await queue.get()
        flow_count += 1

        if flow_count % 10 == 0:
            amount = flow_count * 2.25
            await gos.publish("flow_meter", dumps({"new_flow": amount}))

async def give_water(conn: P2PConnection, peers: set[str], pump: Pump):
    req_resp = conn.get_request_response_service()
    async with pump:
        for peer in peers:
            water_ready = dumps({'water_ready': True})
            try:
                await pump.start()
                response = await req_resp.send_request(peer, water_ready, 'water')
                pump.reduce()
                if len(response.error) > 0:
                    raise Exception(f'{peer}: {response.error}')
            except Exception as e:
                print(f'Error sending water to {peer}: {e}')
                continue
            response_data = loads(response.data)
            if not response_data['done']:
                print(f'{peer} did not accept water')
        await pump.stop()

async def poll_water_claims(conn: P2PConnection):
    gos = conn.get_gossip_sub_service()
    last_time: datetime = datetime.now()
    total_claims = 0
    peers = set()
    pump = Pump()
    async with await gos.subscribe('water_claims') as s:
        async for msg in s:
            data = loads(msg.msg.data)
            peer_id = msg.peer_id
            peers.add(peer_id)
            print(f'Received {data} water claim from {peer_id}')
            total_claims += data['claim']
            time_delta = datetime.now() - last_time
            if time_delta.total_seconds() > OUR_TIME_DELTA:
                asyncio.create_task(give_water(conn, peers, pump))
                last_time = datetime.now()


async def main():
    async with P2PConnection() as conn:
        try:
            await asyncio.gather(
                asyncio.create_task(report_global_counter(conn)),
                asyncio.create_task(poll_water_claims(conn))
            )
        except Exception as e:
            print(f'Error: {e}')

if __name__ == '__main__':
    asyncio.run(main())
