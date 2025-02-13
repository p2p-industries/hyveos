import asyncio
import os
from hyveos_sdk import Connection, PubSubService, RequestResponseService
from json import dumps, loads
from gpiozero import PWMOutputDevice, OutputDevice
from time import time

from hyveos_sdk.protocol.bridge_pb2 import PubSubRecvMessage

FLOW_METER_PIN = int(os.environ['FLOW_METER_PIN'])
PWM_PIN = int(os.environ['PWM_PIN'])
FWD_PIN = int(os.environ['FWD_PIN'])
REV_PIN = int(os.environ['REV_PIN'])


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

    def add_claim(self, msg: PubSubRecvMessage):
        if msg.source.peer_id in self.claims:
            self.claims[msg.source.peer_id] += loads(msg.msg.data.data)['claim']
        else:
            self.claims[msg.source.peer_id] = loads(msg.msg.data.data)['claim']

        self.event.set()

    def clear(self):
        self.claims = {}

    async def __aenter__(self):
        await self.semaphore.acquire()

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.semaphore.release()


async def give_water(
    req_resp: RequestResponseService, water_claims: WaterClaims, pump: Pump
):
    while True:
        async with pump:
            async with water_claims:
                for peer in water_claims.claims.keys():
                    water_ready = dumps({'water_ready': True})
                    try:
                        print('Starting pump')
                        await pump.start()
                        response = await req_resp.send_request(
                            peer, water_ready, 'water'
                        )
                        pump.reduce()
                        if len(response.error) > 0:
                            raise Exception(f'{peer}: {response.error}')
                    except Exception as e:
                        print(f'Error sending water to {peer}: {e}')
                        continue
                    response_data = loads(response.data.data)
                    if not response_data['done']:
                        print(f'{peer} did not accept water')

                water_claims.clear()
            await pump.stop()

        await water_claims.event.wait()
        water_claims.event.clear()


async def monitor_water_claims(pub_sub: PubSubService, water_claims: WaterClaims):
    async with pub_sub.subscribe('watering_request') as messages:
        async for msg in messages:
            async with water_claims:
                print(f'Received water claim from {msg.source}')
                water_claims.add_claim(msg)


async def main():
    async with Connection() as conn:
        pub_sub = conn.get_pub_sub_service()
        req_resp = conn.get_request_response_service()

        water_claims = WaterClaims()
        pump = Pump()

        try:
            await asyncio.gather(
                monitor_water_claims(pub_sub, water_claims),
                give_water(req_resp, water_claims, pump),
            )
        except Exception as e:
            print(f'Error: {e}')


if __name__ == '__main__':
    asyncio.run(main())
