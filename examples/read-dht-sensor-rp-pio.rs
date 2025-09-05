#![no_std]
#![no_main]


use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embassy_dht_sensor::{DHTSensor, DHTSensorError};
use {defmt::info, defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());

    let pio = p.PIO0;


    let Pio {
        mut common,
        sm0,
        irq_flags,
        ..
    } = Pio::new(pio, Irqs);
    let pin = common.make_pio_pin(p.PIN_0);
    let mut dht_sensor = DHTSensor::new(pin, common, sm0, irq_flags);
    loop {
        match dht_sensor.read().await {
            Ok(data) => {
                info!("temperature: {:?}, humidity: {:?}", data.temperature, data.humidity);
            }
            Err(e) => {
                match e {
                    DHTSensorError::Timeout => {
                        info!("Timeout");
                    }
                    DHTSensorError::ChecksumError => {
                        info!("CRC error");
                    }
                    DHTSensorError::InvalidData => {
                        info!("Invalid data");
                    }
                }
            }
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
