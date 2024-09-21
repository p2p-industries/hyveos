import asyncio
import time
import os

from gpiozero import DigitalInputDevice
from p2pindustries.services.db import DBService

MOISTURE_1_PIN = 23
MOISTURE_2_PIN = 8
MOISTURE_3_PIN = 25


class MoistureSensor(object):
    """Grow moisture sensor driver."""

    _gpio_pin: int
    _input: DigitalInputDevice
    _db: DBService
    _loop: asyncio.AbstractEventLoop
    _key: str
    _threshold: int
    _count: int
    _reading: float
    _last_pulse: float
    _new_data: bool
    _time_last_reading: float
    _time_start: float

    @classmethod
    async def create(
        cls,
        channel: int,
        db: DBService,
    ) -> 'MoistureSensor':
        """Create a new moisture sensor.

        Uses an interrupt to count pulses on the GPIO pin corresponding to the selected channel.

        The moisture reading is given as pulses per second.

        :param channel: One of 1, 2 or 3. 4 can optionally be used to set up a sensor on the Int pin (BCM4)
        :param wet_point: Wet point in pulses/sec
        :param dry_point: Dry point in pulses/sec

        """
        self = cls()
        self._gpio_pin = [
            MOISTURE_1_PIN,
            MOISTURE_2_PIN,
            MOISTURE_3_PIN,
        ][channel]

        self._input = DigitalInputDevice(self._gpio_pin, bounce_time=1 / 1000.0)

        self._db = db
        self._loop = asyncio.get_event_loop()

        self._key = f'moisture_threshold_{channel}'
        data = (await db.get(self._key)).data
        if data is not None:
            threshold = int.from_bytes(data.data)
        else:
            threshold = 10
            await db.put(self._key, threshold.to_bytes(1))

        self._threshold = threshold

        self._count = 0
        self._reading = 0
        self._last_pulse = time.monotonic()
        self._new_data = False
        self._time_last_reading = time.monotonic()
        self._input.when_activated = self._event_handler
        self._time_start = time.monotonic()

        return self

    def _event_handler(self, pin):
        self._count += 1
        self._last_pulse = time.monotonic()
        if self._time_elapsed >= 1.0:
            self._reading = self._count / self._time_elapsed
            self._count = 0
            self._time_last_reading = time.monotonic()
            self._new_data = True

    async def increase_threshold(self):
        self._threshold += 1
        await self._db.put(self._key, self._threshold.to_bytes(1))

    async def decrease_threshold(self):
        self._threshold -= 1
        await self._db.put(self._key, self._threshold.to_bytes(1))

    @property
    def _time_elapsed(self):
        return time.monotonic() - self._time_last_reading

    @property
    def moisture(self):
        """Return the raw moisture level.

        The value returned is the pulses/sec read from the soil moisture sensor.

        This value is inversely proportional to the amount of moisture.

        Full immersion in water is approximately 50 pulses/sec.

        Fully dry (in air) is approximately 900 pulses/sec.

        """
        self._new_data = False
        return self._reading

    @property
    def threshold(self):
        return self._threshold

    @property
    def is_dry(self):
        return self.moisture > self._threshold

    @property
    def active(self):
        """Check if the moisture sensor is producing a valid reading."""
        return (
            (time.monotonic() - self._last_pulse) < 1.0
            and self._reading > 0
            and self._reading < 28
        )

    @property
    def new_data(self):
        """Check for new reading.

        Returns True if moisture value has been updated since last reading moisture or saturation.

        """
        return self._new_data
