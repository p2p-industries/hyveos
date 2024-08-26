import asyncio
from gpiozero import LED
from .moisture import MoistureSensor

VALVE_1_PIN = 17
VALVE_2_PIN = 27
VALVE_3_PIN = 22

class ValveController:
    valve: LED
    sensor: MoistureSensor
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def __init__(self, moisture_sensor: MoistureSensor, channel: int = 1):
        pin = [VALVE_1_PIN, VALVE_2_PIN, VALVE_3_PIN][channel - 1]
        self.valve = LED(pin)
        self.sensor = moisture_sensor

    async def open_until_satisfied(self):
        print(f"[ML: {self.sensor.moisture}] Opening valve.")
        if self.sensor.is_dry:
            self.valve.on()

            while self.sensor.is_dry:
                print(f"[ML: {self.sensor.moisture}] Valve remains open.")
                await asyncio.sleep(0.10)

            print(f"[ML: {self.sensor.moisture}] Closing valve.")

            self.valve.off()

    async def __aenter__(self):
        await self.semaphore.acquire()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.semaphore.release()
