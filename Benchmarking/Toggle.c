// toggle_gpio_fast.c
// Fast GPIO toggle using bcm2835 in C

#include <bcm2835.h>
#include <stdio.h>

#define PIN RPI_GPIO_P1_11  // Physical pin 11 = BCM GPIO 17

int main() {
    if (!bcm2835_init()) {
        printf("Failed to initialize bcm2835.\n");
        return 1;
    }

    bcm2835_gpio_fsel(PIN, BCM2835_GPIO_FSEL_OUTP);  // Set as output

    while (1) {
        bcm2835_gpio_set(PIN);    // Set high
        bcm2835_gpio_clr(PIN);    // Set low
    }

    bcm2835_close();  // Not reached
    return 0;
}
