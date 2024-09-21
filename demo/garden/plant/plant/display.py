import asyncio
import time
from threading import Lock
from enum import Enum
from typing import Callable, Sequence, Tuple

from gpiozero import Button
from ltr559 import LTR559
from st7735 import ST7735
from fonts.ttf import Roboto
from PIL import Image, ImageDraw, ImageFont
from .plant import Plant


class ButtonPin(Enum):
    A = 5
    B = 6
    X = 16
    Y = 24


class Position(Enum):
    TOP_LEFT = 0
    TOP_CENTER = 1
    TOP_RIGHT = 2
    MIDDLE_LEFT = 3
    MIDDLE_CENTER = 4
    MIDDLE_RIGHT = 5
    BOTTOM_LEFT = 6
    BOTTOM_CENTER = 7
    BOTTOM_RIGHT = 8


class Color(Enum):
    WHITE = (255, 255, 255)
    RED = (255, 127, 127)
    GREEN = (127, 255, 127)
    BLUE = (127, 127, 255)
    YELLOW = (255, 255, 63)
    CYAN = (63, 255, 255)
    MAGENTA = (255, 63, 255)
    GRAY = (127, 127, 127)
    BLACK = (0, 0, 0)


class State:
    plants: list[Plant]
    light: LTR559
    foo: int

    def __init__(self, plants: list[Plant], light: LTR559):
        self.plants = plants
        self.light = light
        self.foo = 0


class Label:
    position: Position
    make_color: Callable[[State], Color]
    make_text: Callable[[State], str]

    def __init__(
        self,
        position: Position,
        color: Callable[[State], Color] | Color,
        text: Callable[[State], str] | str,
    ):
        self.position = position

        if isinstance(color, Color):
            self.make_color = lambda _: color
        else:
            self.make_color = color

        if isinstance(text, str):
            self.make_text = lambda _: text
        else:
            self.make_text = text


class View:
    def make_title(self, state: State) -> str:
        return 'Menu'

    async def handle_button(self, state: State, pin: ButtonPin) -> bool:
        return False

    def labels(self) -> list[Label]:
        return []


class OverviewView(View):
    _labels: list[Label]

    def __init__(self):
        self._labels = labels = [
            Label(
                Position.TOP_LEFT,
                Color.YELLOW,
                lambda s: f'Light: {s.light.get_lux():.2f} lux',
            ),
            Label(
                Position.MIDDLE_LEFT,
                Color.BLUE,
                'Moisture (Hz):',
            ),
        ] + [
            Label(
                pos,
                lambda s, i=i: Color.RED if s.plants[i].moisture_sensor.is_dry else Color.GREEN,
                lambda s, i=i: f'{s.plants[i].moisture_sensor.moisture:.2f}',
            )
            for i, pos in enumerate(
                [
                    Position.BOTTOM_LEFT,
                    Position.BOTTOM_CENTER,
                    Position.BOTTOM_RIGHT,
                ],
            )
        ]

    def make_title(self, state: State) -> str:
        return 'Plant Overview'

    def labels(self) -> list[Label]:
        return self._labels


class CalibrationView(View):
    _i: int

    def __init__(self, i: int):
        self._i = i

    def make_title(self, state: State) -> str:
        return f'Calibrating Plant {self._i + 1}'

    async def handle_button(self, state: State, pin: ButtonPin) -> bool:
        match pin:
            case ButtonPin.X:
                await state.plants[self._i].moisture_sensor.increase_threshold()
                return True
            case ButtonPin.Y:
                await state.plants[self._i].moisture_sensor.decrease_threshold()
                return True

        return False

    def labels(self) -> list[Label]:
        def make_current_text(s: State) -> str:
            moisture = s.plants[self._i].moisture_sensor.moisture
            is_dry = s.plants[self._i].moisture_sensor.is_dry
            return f'Current: {moisture:.2f} Hz ({"Dry" if is_dry else "Wet"})'

        return [
            Label(Position.TOP_RIGHT, Color.YELLOW, '^'),
            Label(Position.MIDDLE_LEFT, Color.BLUE, 'Dry Point:'),
            Label(
                Position.MIDDLE_RIGHT,
                Color.BLUE,
                lambda s: f'{s.plants[self._i].moisture_sensor.threshold}',
            ),
            Label(
                Position.BOTTOM_LEFT,
                lambda s: Color.RED if s.plants[self._i].moisture_sensor.is_dry else Color.GREEN,
                make_current_text,
            ),
            Label(Position.BOTTOM_RIGHT, Color.YELLOW, 'v'),
        ]


class Display:
    _display: ST7735
    _image: Image.Image
    _draw: ImageDraw.ImageDraw
    _font: ImageFont.FreeTypeFont
    _buttons: list[Button]
    _state: State
    _menus: Sequence[View]
    _current_menu: int
    _last_button_press: float
    _pressed_button: ButtonPin | None
    _button_lock: Lock

    def __init__(self, plants: list[Plant], light: LTR559):
        self._display = ST7735(
            port=0, cs=1, dc=9, backlight=12, rotation=270, spi_speed_hz=80000000
        )

        self._image = Image.new(
            'RGBA', (self._display.width, self._display.height), color=(0, 0, 0)
        )
        self._draw = ImageDraw.Draw(self._image)

        self._font = ImageFont.truetype(Roboto, 14)

        self._buttons = [
            Button(
                pin.value,
                bounce_time=0.1,
            )
            for pin in ButtonPin
        ]

        for button in self._buttons:
            button.when_pressed = self._handle_button

        self._state = State(plants, light)

        self._menus = [
            OverviewView(),
        ] + [CalibrationView(i) for i in range(len(plants))]

        self._current_menu = 0

        self._last_button_press = time.monotonic()
        self._pressed_button = None
        self._button_lock = Lock()

    def _handle_button(self, button: Button):
        if button.pin is None:
            return

        pin = ButtonPin(button.pin.number)

        print(f'Button {pin} pressed')

        with self._button_lock:
            self._pressed_button = pin

    async def run(self):
        self._display.begin()

        while True:
            menu = self._menus[self._current_menu]

            with self._button_lock:
                pressed_button = self._pressed_button
                self._pressed_button = None

            if pressed_button is not None:
                match pressed_button:
                    case ButtonPin.A:
                        print('Switching menu')
                        self._current_menu = (self._current_menu + 1) % len(self._menus)
                        handled = True
                    case _:
                        handled = await menu.handle_button(self._state, pressed_button)

                if handled:
                    self._last_button_press = time.monotonic()

            width = self._display.width
            height = self._display.height

            self._draw.rectangle((0, 0, width, height), (0, 0, 0))

            self._draw.text(
                (2, 2),
                menu.make_title(self._state),
                font=self._font,
                fill=(255, 255, 255),
            )

            for label in menu.labels():
                x = 2
                y = 2
                anchor = 'la'

                match label.position:
                    case Position.TOP_LEFT:
                        x = 2
                        y = 20
                        anchor = 'la'
                    case Position.TOP_CENTER:
                        x = width // 2
                        y = 20
                        anchor = 'ma'
                    case Position.TOP_RIGHT:
                        x = width - 2
                        y = 20
                        anchor = 'ra'
                    case Position.MIDDLE_LEFT:
                        x = 2
                        y = (height - 22) // 2 + 20
                        anchor = 'lm'
                    case Position.MIDDLE_CENTER:
                        x = width // 2
                        y = (height - 22) // 2 + 20
                        anchor = 'mm'
                    case Position.MIDDLE_RIGHT:
                        x = width - 2
                        y = (height - 22) // 2 + 20
                        anchor = 'rm'
                    case Position.BOTTOM_LEFT:
                        x = 2
                        y = height - 2
                        anchor = 'ld'
                    case Position.BOTTOM_CENTER:
                        x = width // 2
                        y = height - 2
                        anchor = 'md'
                    case Position.BOTTOM_RIGHT:
                        x = width - 2
                        y = height - 2
                        anchor = 'rd'

                self._draw.text(
                    (x, y),
                    label.make_text(self._state),
                    fill=label.make_color(self._state).value,
                    font=self._font,
                    anchor=anchor,
                )

            self._display.display(self._image)

            await asyncio.sleep(0.1)

            if (
                self._current_menu != 0
                and time.monotonic() - self._last_button_press > 60
            ):
                print('Returning to main menu')
                self._current_menu = 0
