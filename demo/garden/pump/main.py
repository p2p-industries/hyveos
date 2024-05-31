import asyncio
import os
from asyncio import Queue
from p2pindustries import DHTService, GossipSubService, P2PConnection, RequestResponseService
from json import dumps, loads
from gpiozero import Button, PWMOutputDevice
from datetime import datetime, timedelta
from time import time
from random import randint

FLOW_METER_PIN = int(os.environ['FLOW_METER_PIN'])
PUMP_PIN = int(os.environ['PUMP_PIN'])

OUR_TIME_DELTA = (60 * 2)
#OUR_TIME_DELTA = timedelta(hours=1).total_seconds()


class Pump:
    device: PWMOutputDevice
    running: bool = False
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def __init__(self) -> None:
        self.device = PWMOutputDevice(PUMP_PIN)

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


class WaterClaims:
    claims: dict[str, int] = {}
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def add_claim(self, msg):
        if msg.peer_id in self.claims:
            self.claims[msg.peer_id] += loads(msg.msg.data)['claim']
        else:
            self.claims[msg.peer_id] = loads(msg.msg.data)['claim']

    def clear(self):
        self.claims = {}

    async def __aenter__(self):
        await self.semaphore.acquire()

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.semaphore.release()


async def discover_role_peer(dht, name):
    async with dht.get_providers("identification", name) as providers:
        async for provider in providers:
            return provider.peer_id


async def report_global_counter(conn: P2PConnection):
    gos = conn.get_gossip_sub_service()
    queue = Queue()

    flow_meter = Button(FLOW_METER_PIN)

    def on_click():
        queue.put_nowait(1)

    flow_meter.when_activated = on_click

    flow_count = 0

    prometheus_peer = await discover_role_peer(conn.get_dht_service(), 'prometheus')
    reqres = conn.get_request_response_service()

    while True:
        _ = await queue.get()
        flow_count += 1

        if flow_count % 10 == 0:
            amount = flow_count * 2.25
            await reqres.send_request(prometheus_peer, dumps({'flow_meter': amount}), 'flow_meter')


async def give_water(conn: P2PConnection, water_claims: WaterClaims, pump: Pump):
    req_resp = conn.get_request_response_service()
    while True:
        async with pump:
            async with water_claims:
                for peer in water_claims.claims.keys():
                    water_ready = dumps({'water_ready': True})
                    try:
                        print("Starting pump")
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

        await asyncio.sleep(OUR_TIME_DELTA)


async def monitor_water_claims(conn: P2PConnection, water_claims: WaterClaims):
    gos = conn.get_gossip_sub_service()
    async with await gos.subscribe('watering_request') as messages:
        async for msg in messages:
            async with water_claims:
                print(f"Received water claim from {msg.peer_id}")
                water_claims.add_claim(msg)


async def main():
    async with P2PConnection() as conn:
        water_claims = WaterClaims()
        pump = Pump()

        try:
            await asyncio.gather(
                report_global_counter(conn),
                monitor_water_claims(conn, water_claims),
                give_water(conn, water_claims, pump)
            )
        except Exception as e:
            print(f'Error: {e}')

if __name__ == '__main__':
    asyncio.run(main())
