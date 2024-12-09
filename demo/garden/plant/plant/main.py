from enum import Enum
import os
import json
import asyncio
from typing import Callable
import ltr559
from hyveos_sdk import Connection
from hyveos_sdk.services.dht import DHTService
from hyveos_sdk.services.gossip_sub import GossipSubService
from hyveos_sdk.services.request_response import RequestResponseService
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
    gos: GossipSubService, name, generator: Callable[[], float], metric_type: MetricType
):
    while True:
        value = generator()
        TOPIC = 'export_data'
        payload = json.dumps(
            {'name': name, 'value': value, 'metric_type': metric_type.value}
        )
        await gos.publish(payload, TOPIC)
        await asyncio.sleep(DATA_PUBLISH_INTERVAL)


async def manage_watering_request(gos: GossipSubService, peer_id, plants: list[Plant]):
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
                    print(f'Sending watering request')
                    await gos.publish(
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


async def manage_valve(reqres: RequestResponseService, plants: list[Plant], peer_id):
    global pending_watering_request

    async with reqres.receive('water') as requests:
        async for request in requests:
            if json.loads(request.msg.data.data)['water_ready']:
                async with lock:
                    if pending_watering_request != None:
                        plant = plants[pending_watering_request]
                        async with plant.valve_controller:
                            await plant.valve_controller.open_until_satisfied()
                            await reqres.respond(
                                request.seq,
                                json.dumps({'peer_id': peer_id, 'done': True}),
                            )
                            pending_watering_request = None
                    else:
                        await reqres.respond(
                            request.seq, '', f'Peer {peer_id} did not request water'
                        )


async def control_soil_moisture(
    gos: GossipSubService, reqres: RequestResponseService, plants: list[Plant], peer_id
):
    await asyncio.gather(
        asyncio.create_task(manage_valve(reqres, plants, peer_id)),
        asyncio.create_task(manage_watering_request(gos, peer_id, plants)),
    )


async def discover_role_peer(dht: DHTService, name):
    async with dht.get_providers('identification', name) as providers:
        async for provider in providers:
            return provider.peer_id


async def register_plant(dht: DHTService):
    await dht.provide('identification', 'plant')


async def main():
    async with Connection() as connection:
        discovery = connection.get_discovery_service()
        reqres = connection.get_request_response_service()
        gos = connection.get_gossip_sub_service()
        dht = connection.get_dht_service()
        db = connection.get_db_service()

        plants: list[Plant] = [await Plant.create(i, db) for i in range(3)]

        light = ltr559.LTR559()

        display = Display(plants, light)

        my_peer_id = await discovery.get_own_id()
        _prometheus_peer = await discover_role_peer(dht, 'prometheus')
        await register_plant(dht)

        plant_data_tasks = [
            write_to_prometheus(
                gos,
                f'moisture_{i}_{my_peer_id}',
                lambda: plant.moisture_sensor.moisture,
                MetricType.GAUGE,
            )
            for i, plant in enumerate(plants, start=1)
        ]

        await asyncio.gather(
            asyncio.create_task(control_soil_moisture(gos, reqres, plants, my_peer_id)),
            asyncio.create_task(display.run()),
            asyncio.create_task(
                write_to_prometheus(
                    gos, f'light_{my_peer_id}', light.get_lux, MetricType.GAUGE
                )
            ),
            *plant_data_tasks,
        )


if __name__ == '__main__':
    asyncio.run(main())
