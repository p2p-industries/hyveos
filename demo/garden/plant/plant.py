import os
import board
import json
import asyncio
import adafruit_bh1750
import adafruit_ahtx0
from gpiozero import MCP3008, OutputDevice, LED
from p2pindustries import P2PConnection

MOISTURE_THRESHOLD = 0.7
DATA_PUBLISH_INTERVAL = 2 * 60
VALVE_PIN = 8

pending_watering_request = False
lock = asyncio.Lock()


class ValveController:
    valve: LED
    sensor: MCP3008
    semaphore: asyncio.Semaphore = asyncio.Semaphore(1)

    def __init__(self, valve, moisture_sensor):
        self.vale = valve
        self.sensor = moisture_sensor

    async def open_until_satisfied(self):
        if self.sensor.value < MOISTURE_THRESHOLD:
            self.valve.on()

            while self.sensor.value < MOISTURE_THRESHOLD:
                await asyncio.sleep(0.10)

            self.valve.off()

    def __aenter__(self):
        self.semaphore.acquire()
        return self

    def __aexit__(self, exc_type, exc_val, exc_tb):
        self.semaphore.release()


async def write_to_prometheus(reqres, prometheus_peer, topic, generator):
    while True:
        await reqres.send_request(prometheus_peer, generator(), topic)
        await asyncio.sleep(DATA_PUBLISH_INTERVAL)


async def manage_watering_request(gossip, peer_id, moisture_sensor):
    global pending_watering_request

    while True:
        async with lock:
            if (not pending_watering_request) and (moisture_sensor.value < MOISTURE_THRESHOLD):
                await gossip.publish(json.dumps({
                    "peer_id": peer_id,
                    "watering_request": True
                }), "watering_request")

                pending_watering_request = True

        await asyncio.sleep(60)


async def manage_valve(reqres, valve_controller, peer_id):
    global pending_watering_request

    async with reqres.receive('water') as requests:
        async for request in requests:
            if json.loads(request.msg.data)["water_ready"]:
                async with lock:
                    if pending_watering_request:
                        async with valve_controller:
                            await valve_controller.open_until_satisfied()
                            await reqres.send_response(request.seq, json.dumps({'peer_id': peer_id, 'done': True}))
                            pending_watering_request = False
                    else:
                        await reqres.send_response(request.seq, '', f'Peer {peer_id} did not request water')


async def control_soil_moisture(gossip, reqres, soil_sensor, peer_id):
    valve_controller = ValveController(LED(int(os.environ["VALVE_PIN"])), soil_sensor)

    await asyncio.gather(
        manage_valve(reqres, valve_controller, peer_id),
        manage_watering_request(gossip, peer_id, soil_sensor),
    )


async def discover_role_peer(dht, name):
    async with dht.get_providers("identification", name) as providers:
        async for provider in providers:
            return provider.peer_id


async def register_plant(dht):
    await dht.provide("identification", "plant")


async def main():
    async with P2PConnection() as connection:
        discovery = connection.get_discovery_service()
        reqres = connection.get_request_response_service()
        gsub = connection.get_gossip_sub_service()
        dht = connection.get_dht_service()

        my_peer_id = await discovery.get_own_id()
        prometheus_peer = await discover_role_peer(dht, "prometheus")
        await register_plant(dht)

        i2c = board.I2C()
        light_sensor = adafruit_bh1750.BH1750(i2c)
        climate_sensor = adafruit_ahtx0.AHTx0(i2c)
        moisture = MCP3008(channel=0)

        await asyncio.gather(
            write_to_prometheus(reqres, prometheus_peer, 'temperature',
                                lambda: json.dumps({'peer_id': my_peer_id, 'temperature': climate_sensor.temperature})),
            write_to_prometheus(reqres, prometheus_peer, 'humidity',
                                lambda: json.dumps(
                                    {'peer_id': my_peer_id, 'humidity': climate_sensor.relative_humidity})),
            write_to_prometheus(reqres, prometheus_peer, 'lux',
                                lambda: json.dumps({'peer_id': my_peer_id, 'lux': light_sensor.lux})),
            write_to_prometheus(reqres, prometheus_peer, 'soil_moisture',
                                lambda: json.dumps({'peer_id': my_peer_id, 'soil_moisture': moisture.value})),
            control_soil_moisture(gsub, reqres, moisture, my_peer_id))


if __name__ == '__main__':
    asyncio.run(main())
