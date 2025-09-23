# Embassy DHT Sensor Library

This Rust library provides an interface for interacting with DHT1X and DHT2X temperature and humidity sensors using the Embassy framework.

Adafruit DHT sensor library is used as a reference for this library.
https://github.com/adafruit/DHT-sensor-library

## Note
This library should be used in **release** mode. The measurements made in the **debug** mode are not accurate enough.

### PIO support
Since version 0.2.2 this library supports PIO (Programmable Input/Output) for Raspberry Pi Pico, which allows for more accurate timing when reading data from the DHT sensors.
To enable PIO support, make sure to include the `pio` feature in your `Cargo.toml`. Using PIO is recommended for better performance.
There are no requirements to run the library in release mode when using PIO.

```toml

## Supported Devices
Currently only the Raspberry Pi Pico board is supported.

## Getting Started

### Installation

Add `embassy-dht-sensor` to your `Cargo.toml`:

```toml
[dependencies]
embassy-dht-sensor = "0.1.0"
```

## Usage
Initialize your Raspberry Pi Pico board with Embassy.
Create an instance of DHTSensor with the GPIO pin connected to your DHT sensor.
Use the read method to get temperature and humidity readings.

### Example using PIO (recommended):

```rust
#![no_std]
#![no_main]


use embassy_time::Instant;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embassy_dht_sensor::{DHTSensor, DHTSensorError};
use {defmt, defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());

    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0,
        ..
    } = Pio::new(pio, Irqs);
    let pin = common.make_pio_pin(p.PIN_0);

    let mut dht_sensor = DHTSensor::new(pin, common, sm0);
    loop {
        match dht_sensor.read().await {
            Ok(data) => {
                info!("Temperature: {:?}, Humidity: {:?}", data.temperature, data.humidity);
            }
            Err(e) => {
                info!("Error reading from DHT sensor: {:?}", e);
            }
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
```

### Example using legacy GPIO (non-PIO):

```rust
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Flex};
use embassy_time::{Duration, Timer};
use embassy_dht_sensor::DHTSensor;
use defmt::info;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
let p = embassy_rp::init(Default::default());
let pin = Flex::new(AnyPin::from(p.PIN_0));
let mut dht_sensor = DHTSensor::new(pin);

    loop {
        match dht_sensor.read() {
            Ok(data) => {
                info!("Temperature: {:?}, Humidity: {:?}", data.temperature, data.humidity);
            },
            Err(e) => {
                info!("Error reading from DHT sensor: {:?}", e);
            }
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
'''


