#![no_std]
#![no_main]


use embassy_time::Instant;
use embassy_rp::peripherals::USB;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use embassy_dht_sensor::{DHTSensor, DHTSensorError};
use {defmt, defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    USBCTRL_IRQ =>  embassy_rp::usb::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    Timer::after(Duration::from_secs(3)).await;

    info!("DHT sensor example {}", Instant::now().as_millis());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0,
        ..
    } = Pio::new(pio, Irqs);
    let pin = common.make_pio_pin(p.PIN_0);

    let mut dht_sensor = DHTSensor::new(pin, common, sm0);
    Timer::after(Duration::from_secs(3)).await;
    loop {
        match dht_sensor.read().await {
            Ok(data) => {
                info!("temperature: {:?}, humidity: {:?}", data.temperature, data.humidity);
                log::info!("temperature: {:.2} °C, humidity: {:.2} %", data.temperature, data.humidity);
            }
            Err(e) => {
                match e {
                    DHTSensorError::NoData => {
                        info!("NoData");
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

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
