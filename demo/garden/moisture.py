from gpiozero import MCP3008
from time import sleep

moisture = MCP3008(channel=0)

while True:
    print(f"Moisture: {moisture.value}")
    sleep(0.1)