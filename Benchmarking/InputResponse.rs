// input_output_gpio_test.rs
// Mirror input GPIO 27 to output GPIO 17

use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::error::Error;
use std::thread;
use std::time::Duration;

const INPUT_GPIO: u8 = 27;  // Input pin
const OUTPUT_GPIO: u8 = 17; // Output pin

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;

    let input: InputPin = gpio.get(INPUT_GPIO)?.into_input();
    let mut output: OutputPin = gpio.get(OUTPUT_GPIO)?.into_output();

    loop {
        if input.is_high() {
            output.set_high();
        } else {
            output.set_low();
        }

        // Optionally reduce CPU usage
        //thread::sleep(Duration::from_micros(10));
    }
}
