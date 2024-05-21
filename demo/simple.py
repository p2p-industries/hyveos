from gpiozero import Button, LED
from signal import pause
from time import sleep

led = LED(17)
switch = Button(27)

switch.when_pressed = lambda: led.on()
switch.when_released = lambda: led.off()

pause()