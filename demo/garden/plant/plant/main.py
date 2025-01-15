from enum import Enum
import os
import json
import asyncio
from typing import Callable
import ltr559
from hyveos_sdk import Connection, DiscoveryService, PubSubService, RequestResponseService
from .display import Display
from .moisture import MoistureSensor
from .plant import Plant
from .valve import ValveController

MOISTURE_THRESHOLD = 10.0
DATA_PUBLISH_INTERVAL = 2 * 60

pending_watering_request: int | None = None
lock = asyncio.Lock()


class MetricType(Enum):
    COUNTER = 'Counter'
    GAUGE = 'Gauge'


async def write_to_prometheus(
    pub_sub: PubSubService, name, generator: Callable[[], float], metric_type: MetricType
):
    while True:
        value = generator()
        TOPIC = 'export_data'
        payload = json.dumps(
            {'name': name, 'value': value, 'metric_type': metric_type.value}
        )
        await pub_sub.publish(payload, TOPIC)
        await asyncio.sleep(DATA_PUBLISH_INTERVAL)


async def manage_watering_request(pub_sub: PubSubService, peer_id, plants: list[Plant]):
    global pending_watering_request

    while True:
        for i, plant in enumerate(plants):
            async with lock:
                if (pending_watering_request == None) and (
                    plant.moisture_sensor.is_dry
                ):
                    print(
                        f'Moisture level of plant {i + 1} above threshold: {plant.moisture_sensor.moisture:.2f} Hz'
                    )
                    print('Sending watering request')
                    await pub_sub.publish(
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


async def manage_valve(req_res: RequestResponseService, plants: list[Plant], peer_id):
    global pending_watering_request

    async with req_res.receive('water') as requests:
        async for request in requests:
            if json.loads(request.msg.data.data)['water_ready']:
                async with lock:
                    if pending_watering_request != None:
                        plant = plants[pending_watering_request]
                        async with plant.valve_controller:
                            await plant.valve_controller.open_until_satisfied()
                            await req_res.respond(
                                request.seq,
                                json.dumps({'peer_id': peer_id, 'done': True}),
                            )
                            pending_watering_request = None
                    else:
                        await req_res.respond(
                            request.seq, '', f'Peer {peer_id} did not request water'
                        )


async def control_soil_moisture(
    pub_sub: PubSubService, req_res: RequestResponseService, plants: list[Plant], peer_id
):
    await asyncio.gather(
        asyncio.create_task(manage_valve(req_res, plants, peer_id)),
        asyncio.create_task(manage_watering_request(pub_sub, peer_id, plants)),
    )


async def discover_role_peer(discovery: DiscoveryService, name):
    async with discovery.get_providers('identification', name) as providers:
        async for provider in providers:
            return provider.peer_id


async def register_plant(discovery: DiscoveryService):
    await discovery.provide('identification', 'plant')


async def main():
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        req_res = connection.get_request_response_service()
        pub_sub = connection.get_pub_sub_service()
        local_kv = connection.get_local_kv_service()

        plants: list[Plant] = [await Plant.create(i, local_kv) for i in range(3)]

        light = ltr559.LTR559()

        display = Display(plants, light)

        my_peer_id = await connection.get_id()
        await register_plant(discovery)

        plant_data_tasks = [
            write_to_prometheus(
                pub_sub,
                f'moisture_{i}_{my_peer_id}',
                lambda: plant.moisture_sensor.moisture,
                MetricType.GAUGE,
            )
            for i, plant in enumerate(plants, start=1)
        ]

        await asyncio.gather(
            asyncio.create_task(control_soil_moisture(pub_sub, req_res, plants, my_peer_id)),
            asyncio.create_task(display.run()),
            asyncio.create_task(
                write_to_prometheus(
                    pub_sub, f'light_{my_peer_id}', light.get_lux, MetricType.GAUGE
                )
            ),
            *plant_data_tasks,
        )


if __name__ == '__main__':
    asyncio.run(main())
