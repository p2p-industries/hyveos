import time
import os

from gpiozero import DigitalInputDevice

MOISTURE_1_PIN = 23
MOISTURE_2_PIN = 8
MOISTURE_3_PIN = 25


MOISTURE_THRESHOLDS = [
    float(os.environ.get(f"MOISTURE_THRESHOLD_{i}", "10.0")) for i in range(1, 4)
]


class MoistureSensor(object):
    """Grow moisture sensor driver."""

    def __init__(
        self,
        channel=1,
        wet_point=None,
        dry_point=None,
    ):
        """Create a new moisture sensor.

        Uses an interrupt to count pulses on the GPIO pin corresponding to the selected channel.

        The moisture reading is given as pulses per second.

        :param channel: One of 1, 2 or 3. 4 can optionally be used to set up a sensor on the Int pin (BCM4)
        :param wet_point: Wet point in pulses/sec
        :param dry_point: Dry point in pulses/sec

        """
        self._gpio_pin = [
            MOISTURE_1_PIN,
            MOISTURE_2_PIN,
            MOISTURE_3_PIN,
        ][channel - 1]

        self._input = DigitalInputDevice(self._gpio_pin, bounce_time=1 / 1000.0)

        self._threshold = MOISTURE_THRESHOLDS[channel - 1]

        self._count = 0
        self._reading = 0
        self._history = []
        self._history_length = 200
        self._last_pulse = time.time()
        self._new_data = False
        self._wet_point = wet_point if wet_point is not None else 0.7
        self._dry_point = dry_point if dry_point is not None else 27.6
        self._time_last_reading = time.time()
        self._input.when_activated = self._event_handler
        self._time_start = time.time()

    def _event_handler(self, pin):
        self._count += 1
        self._last_pulse = time.time()
        if self._time_elapsed >= 1.0:
            self._reading = self._count / self._time_elapsed
            self._history.insert(0, self._reading)
            self._history = self._history[: self._history_length]
            self._count = 0
            self._time_last_reading = time.time()
            self._new_data = True

    @property
    def history(self):
        history = []

        for moisture in self._history:
            saturation = float(moisture - self._dry_point) / self.range
            saturation = round(saturation, 3)
            history.append(max(0.0, min(1.0, saturation)))

        return history

    @property
    def _time_elapsed(self):
        return time.time() - self._time_last_reading

    def set_wet_point(self, value=None):
        """Set the sensor wet point.

        This is the watered, wet state of your soil.

        It should be set shortly after watering. Leave ~5 mins for moisture to permeate.

        :param value: Wet point value to set in pulses/sec, leave as None to set the last sensor reading.

        """
        self._wet_point = value if value is not None else self._reading

    def set_dry_point(self, value=None):
        """Set the sensor dry point.

        This is the unwatered, dry state of your soil.

        It should be set when the soil is dry to the touch.

        :param value: Dry point value to set in pulses/sec, leave as None to set the last sensor reading.

        """
        self._dry_point = value if value is not None else self._reading

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
    def is_dry(self):
        return self.moisture > self._threshold

    @property
    def active(self):
        """Check if the moisture sensor is producing a valid reading."""
        return (
            (time.time() - self._last_pulse) < 1.0
            and self._reading > 0
            and self._reading < 28
        )

    @property
    def new_data(self):
        """Check for new reading.

        Returns True if moisture value has been updated since last reading moisture or saturation.

        """
        return self._new_data

    @property
    def range(self):
        """Return the range sensor range (wet - dry points)."""
        return self._wet_point - self._dry_point

    @property
    def saturation(self):
        """Return saturation as a float from 0.0 to 1.0.

        This value is calculated using the wet and dry points.

        """
        saturation = float(self.moisture - self._dry_point) / self.range
        saturation = round(saturation, 3)
        return max(0.0, min(1.0, saturation))
