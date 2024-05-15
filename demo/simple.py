from gpiozero import Button, LED
from signal import pause
from time import sleep

led = LED(17)
switch = Button(27)

def switch_on():
    led.on()

def switch_off():
    led.off()

switch.when_pressed = switch_on
switch.when_released = switch_off

pause()