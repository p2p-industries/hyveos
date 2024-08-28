from enum import Enum
import os
import json
import asyncio
from typing import Callable
import ltr559
import st7735
from PIL import Image, ImageDraw, ImageFont
from fonts.ttf import Roboto
from p2pindustries import P2PConnection
from .moisture import MoistureSensor
from .valve import ValveController

MOISTURE_THRESHOLD = 10.0
DATA_PUBLISH_INTERVAL = 2 * 60

pending_watering_request: int | None = None
lock = asyncio.Lock()


class Plant:
    def __init__(self, channel: int = 1, wet_point=None, dry_point=None):
        self.channel = channel
        self.moisture_sensor = MoistureSensor(channel, wet_point, dry_point)
        self.valve_controller = ValveController(self.moisture_sensor, channel)


class MetricType(Enum):
    COUNTER = 'Counter'
    GAUGE = 'Gauge'


async def write_to_prometheus(
    gossip, name, generator: Callable[[], float], metric_type: MetricType
):
    while True:
        value = generator()
        TOPIC = 'export_data'
        payload = json.dumps(
            {'name': name, 'value': value, 'metric_type': metric_type.value}
        )
        await gossip.publish(payload, TOPIC)
        await asyncio.sleep(DATA_PUBLISH_INTERVAL)


async def manage_watering_request(gossip, peer_id, plants: dict[int, Plant]):
    global pending_watering_request

    while True:
        for i, plant in plants.items():
            async with lock:
                if (pending_watering_request == None) and (
                    plant.moisture_sensor.is_dry
                ):
                    print(
                        f'Moisture level {plant.moisture_sensor.moisture} above threshold'
                    )
                    print(f'Sending watering request')
                    await gossip.publish(
                        json.dumps(
                            {
                                'peer_id': peer_id,
                                'watering_request': True,
                                'claim': 0,
                            }
                        ),
                        'watering_request',
                    )

                    pending_watering_request = i

            await asyncio.sleep(1)


async def manage_valve(reqres, plants: dict[int, Plant], peer_id):
    global pending_watering_request

    async with reqres.receive('water') as requests:
        async for request in requests:
            if json.loads(request.msg.data)['water_ready']:
                async with lock:
                    if pending_watering_request != None:
                        plant = plants[pending_watering_request]
                        async with plant.valve_controller:
                            await plant.valve_controller.open_until_satisfied()
                            await reqres.respond(
                                request.seq,
                                json.dumps({'peer_id': peer_id, 'done': True}),
                            )
                            pending_watering_request = False
                    else:
                        await reqres.send_response(
                            request.seq, '', f'Peer {peer_id} did not request water'
                        )


async def control_soil_moisture(gossip, reqres, plants: dict[int, Plant], peer_id):
    await asyncio.gather(
        asyncio.create_task(manage_valve(reqres, plants, peer_id)),
        asyncio.create_task(manage_watering_request(gossip, peer_id, plants)),
    )


async def control_display(light, peer_id: str, plants: dict[int, Plant]):
    display = st7735.ST7735(
        port=0, cs=1, dc=9, backlight=12, rotation=270, spi_speed_hz=80000000
    )
    display.begin()

    image = Image.new('RGBA', (display.width, display.height), color=(0, 0, 0))
    draw = ImageDraw.Draw(image)

    font = ImageFont.truetype(Roboto, 16)
    small_font = ImageFont.truetype(Roboto, 8)

    while True:
        for i, plant in plants.items():
            moisture = plant.moisture_sensor
            draw.rectangle((0, 0, display.width, display.height), (0, 0, 0))
            draw.text((2, 2), f'Node: plant {i}', font=font, fill=(255, 255, 255))
            draw.text(
                (2, 22),
                f'Moisture: {moisture.moisture:.2f}',
                font=font,
                fill=(128, 128, 255),
            )
            draw.text(
                (2, 42), f'Lux: {light.get_lux():.2f}', font=font, fill=(255, 255, 64)
            )
            draw.text(
                (2, 62),
                f'Peer Id: {peer_id}',
                font=small_font,
                fill=(64, 255, 255),
            )

            display.display(image)

            await asyncio.sleep(1)


async def discover_role_peer(dht, name):
    async with dht.get_providers('identification', name) as providers:
        async for provider in providers:
            return provider.peer_id


async def register_plant(dht):
    await dht.provide('identification', 'plant')


async def main():
    plants: dict[int, Plant] = {}
    if plant1 := os.environ.get('PLANT_1'):
        plants[int(plant1)] = Plant(1)
    if plant2 := os.environ.get('PLANT_2'):
        plants[int(plant2)] = Plant(2)
    if plant3 := os.environ.get('PLANT_3'):
        plants[int(plant3)] = Plant(3)

    light = ltr559.LTR559()

    async with P2PConnection() as connection:
        discovery = connection.get_discovery_service()
        reqres = connection.get_request_response_service()
        gsub = connection.get_gossip_sub_service()
        dht = connection.get_dht_service()

        my_peer_id = await discovery.get_own_id()
        _prometheus_peer = await discover_role_peer(dht, 'prometheus')
        await register_plant(dht)

        plant_data_tasks = [
            write_to_prometheus(
                gsub,
                f'moisture_{i}_{my_peer_id}',
                lambda: 1.0 / plant.moisture_sensor.moisture,
                MetricType.GAUGE,
            )
            for i, plant in plants.items()
        ]

        await asyncio.gather(
            asyncio.create_task(
                control_soil_moisture(gsub, reqres, plants, my_peer_id)
            ),
            asyncio.create_task(control_display(light, my_peer_id, plants)),
            asyncio.create_task(
                write_to_prometheus(
                    gsub, f'light_{my_peer_id}', light.get_lux, MetricType.GAUGE
                )
            ),
            *plant_data_tasks,
        )


if __name__ == '__main__':
    asyncio.run(main())
