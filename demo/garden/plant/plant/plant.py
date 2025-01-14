from hyveos_sdk import LocalKVService
from .moisture import MoistureSensor
from .valve import ValveController


class Plant:
    channel: int
    moisture_sensor: MoistureSensor
    valve_controller: ValveController

    @classmethod
    async def create(cls, channel: int, local_kv: LocalKVService) -> 'Plant':
        self = cls()
        self.channel = channel
        self.moisture_sensor = await MoistureSensor.create(channel, local_kv)
        self.valve_controller = ValveController(self.moisture_sensor, channel)
        return self
