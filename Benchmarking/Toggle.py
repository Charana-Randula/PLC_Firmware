# toggle_gpio_fast.py
# Rapid GPIO toggle test using RPi.GPIO (Python)
import RPi.GPIO as GPIO
import time

PIN = 17  # BCM GPIO 17

GPIO.setmode(GPIO.BCM)
GPIO.setup(PIN, GPIO.OUT)

try:
    while True:
        GPIO.output(PIN, GPIO.HIGH)
        GPIO.output(PIN, GPIO.LOW)
except KeyboardInterrupt:
    pass
finally:
    GPIO.cleanup()
