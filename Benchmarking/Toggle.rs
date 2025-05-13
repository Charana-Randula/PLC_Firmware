// toggle_gpio_fast.rs
// Rapid GPIO toggle using rppal crate in Rust
// Use rppal in cargo.toml

use rppal::gpio::Gpio;
use std::error::Error;

const GPIO_PIN: u8 = 17;

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(GPIO_PIN)?.into_output();

    loop {
        pin.set_high();
        pin.set_low();
    }
}
