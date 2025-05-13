# input_output_gpio_test.py
# Set GPIO 17 HIGH if GPIO 27 is HIGH

import RPi.GPIO as GPIO
import time

INPUT_PIN = 27   # BCM GPIO 27 (input)
OUTPUT_PIN = 17  # BCM GPIO 17 (output)

GPIO.setmode(GPIO.BCM)
GPIO.setup(INPUT_PIN, GPIO.IN)
GPIO.setup(OUTPUT_PIN, GPIO.OUT)

try:
    while True:
        if GPIO.input(INPUT_PIN) == GPIO.HIGH:
            GPIO.output(OUTPUT_PIN, GPIO.HIGH)
        else:
            GPIO.output(OUTPUT_PIN, GPIO.LOW)
except KeyboardInterrupt:
    pass
finally:
    GPIO.cleanup()
