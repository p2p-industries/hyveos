from p2pindustries.services.db import DBService
from .moisture import MoistureSensor
from .valve import ValveController


class Plant:
    channel: int
    moisture_sensor: MoistureSensor
    valve_controller: ValveController

    @classmethod
    async def create(cls, channel: int, db: DBService) -> 'Plant':
        self = cls()
        self.channel = channel
        self.moisture_sensor = await MoistureSensor.create(channel, db)
        self.valve_controller = ValveController(self.moisture_sensor, channel)
        return self
