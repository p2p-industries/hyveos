import asyncio
import os
from p2pindustries import P2PConnection
from json import dumps, loads
from gpiozero import PWMOutputDevice, OutputDevice
from time import time

from p2pindustries.protocol.script_pb2 import GossipSubRecvMessage

FLOW_METER_PIN = int(os.environ["FLOW_METER_PIN"])
PWM_PIN = int(os.environ["PWM_PIN"])
FWD_PIN = int(os.environ["FWD_PIN"])
REV_PIN = int(os.environ["REV_PIN"])


OUR_TIME_DELTA = 60 * 2
# OUR_TIME_DELTA = timedelta(hours=1).total_seconds()

ASSUMED_FLOW_RATE = 100.0 / 60.0  # ml/s


class Pump:
    device: PWMOutputDevice
    fwd: OutputDevice
    rev: OutputDevice
    running: bool = False
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def __init__(self) -> None:
        self.device = PWMOutputDevice(PWM_PIN)
        self.fwd = OutputDevice(FWD_PIN)
        self.rev = OutputDevice(REV_PIN)

    def __forward__(self):
        self.fwd.on()
        self.rev.off()

    def __stop__(self):
        self.fwd.off()
        self.rev.off()

    async def __turn_up__(self):
        self.__forward__()
        self.running = True
        start = time()
        self.device.on()
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
        self.__stop__()
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
    event: asyncio.Event = asyncio.Event()

    def add_claim(self, msg: GossipSubRecvMessage):
        if msg.source.peer_id in self.claims:
            self.claims[msg.source.peer_id] += loads(msg.msg.data)["claim"]
        else:
            self.claims[msg.source.peer_id] = loads(msg.msg.data)["claim"]

        self.event.set()

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


async def give_water(conn: P2PConnection, water_claims: WaterClaims, pump: Pump):
    req_resp = conn.get_request_response_service()
    while True:
        async with pump:
            async with water_claims:
                for peer in water_claims.claims.keys():
                    water_ready = dumps({"water_ready": True})
                    try:
                        print("Starting pump")
                        await pump.start()
                        response = await req_resp.send_request(
                            peer, water_ready, "water"
                        )
                        pump.reduce()
                        if len(response.error) > 0:
                            raise Exception(f"{peer}: {response.error}")
                    except Exception as e:
                        print(f"Error sending water to {peer}: {e}")
                        continue
                    response_data = loads(response.data)
                    if not response_data["done"]:
                        print(f"{peer} did not accept water")

                water_claims.clear()
            await pump.stop()

        await water_claims.event.wait()
        water_claims.event.clear()


async def monitor_water_claims(conn: P2PConnection, water_claims: WaterClaims):
    gos = conn.get_gossip_sub_service()
    async with await gos.subscribe("watering_request") as messages:
        async for msg in messages:
            async with water_claims:
                print(f"Received water claim from {msg.source}")
                water_claims.add_claim(msg)


async def main():
    async with P2PConnection() as conn:
        water_claims = WaterClaims()
        pump = Pump()

        try:
            await asyncio.gather(
                monitor_water_claims(conn, water_claims),
                give_water(conn, water_claims, pump),
            )
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())
