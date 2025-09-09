use crate::DHTSensorError::InvalidData;
use crate::{DHTSensorError, DTHResponse};
use embassy_rp::pio::program::{pio_file};
use embassy_rp::pio::{Common, FifoJoin, Instance, Pin, StateMachine};
use embassy_time::Duration;
use fixed::prelude::ToFixed;
use crate::dht::START_LOW_INTERVAL_US;

pub struct DHTSensor<'a, PIO: Instance, const SM: usize> {
    pio: Common<'a, PIO>,
    sm: StateMachine<'a, PIO, SM>,
    data_pin: Pin<'a, PIO>,
    last_response: Option<DTHResponse>,
    last_read_time: Option<embassy_time::Instant>,
    initialized: bool,
}

impl<'a, PIO: Instance, const SM: usize> DHTSensor<'a, PIO, SM> {
    pub fn new(
        data_pin: Pin<'a, PIO>,
        pio: Common<'a, PIO>,
        sm: StateMachine<'a, PIO, SM>,
    ) -> Self {
        DHTSensor {
            pio,
            sm,
            data_pin,
            last_response: None,
            last_read_time: None,
            initialized: false,
        }
    }

    async fn read_raw_data(&mut self) -> Result<[u16; 2], DHTSensorError> {
        if !self.initialized {
            let prg = pio_file!("src/dht22.pio");
            let mut cfg = embassy_rp::pio::Config::default();

            cfg.use_program(&self.pio.load_program(&prg.program), &[]);

            cfg.set_set_pins(&[&self.data_pin]);
            cfg.set_in_pins(&[&self.data_pin]);
            cfg.set_jmp_pin(&self.data_pin);

            cfg.clock_divider = 416.666667f32.to_fixed(); // 300KHz at 125 MHz system clock
            cfg.fifo_join = FifoJoin::Duplex;

            cfg.shift_in = embassy_rp::pio::ShiftConfig {
                threshold: 32,
                direction: embassy_rp::pio::ShiftDirection::Left,
                auto_fill: true,
            };
            self.sm.set_pin_dirs(embassy_rp::pio::Direction::Out, &[&self.data_pin]);
            self.sm.set_config(&cfg);
            self.initialized = true;
        }

        self.sm.set_enable(true);
        self.sm.tx().push((START_LOW_INTERVAL_US as f32 * 0.333) as u32);  // 1 cycle = 3.33us at 300KHz
        let data =self.sm.rx().wait_pull().await;
        let checksum_data = self.sm.rx().wait_pull().await as u8;
        let humidity_data = u16::from((data >> 16) as u16);
        let temperature_data = u16::from((data & 0xffff) as u16);
        self.sm.set_enable(false);

        // Calculate checksum
        let humidity_bytes: [u8; 2] = humidity_data.to_le_bytes();
        let temperature_bytes: [u8; 2] = temperature_data.to_le_bytes();
        let checksum = ((humidity_bytes[0] as u16
            + humidity_bytes[1] as u16
            + temperature_bytes[0] as u16
            + temperature_bytes[1] as u16)
            & 0xff) as u8;
        if checksum_data == checksum {
            Ok([humidity_data, temperature_data])
        } else {
            Err(DHTSensorError::ChecksumError)
        }
    }

    pub async fn read(&mut self) -> Result<DTHResponse, DHTSensorError> {
        let now = embassy_time::Instant::now();
        if let Some(last_read_time) = self.last_read_time {
            if now - last_read_time < Duration::from_secs(crate::dht::MIN_REQUEST_INTERVAL_SECS) {
                if let Some(response) = &self.last_response {
                    return Ok(response.clone());
                }
            }
        }
        else {
            if now.as_secs() < crate::dht::MIN_REQUEST_INTERVAL_SECS {
                return Err(DHTSensorError::NoData);
            }
        }
        match self.read_raw_data().await {
            Ok(data) => {
                let humidity = dht::humidity(&data[0]);
                let temperature = dht::temperature(&data[1]);
                if humidity <= 100.0 {
                    let response = DTHResponse {
                        humidity,
                        temperature,
                    };
                    self.last_response = Some(response.clone());
                    self.last_read_time = Some(embassy_time::Instant::now());
                    Ok(response)
                } else {
                    if let Some(response) = &self.last_response {
                        Ok(response.clone())
                    } else {
                        Err(InvalidData)
                    }
                }
            }
            Err(e) => {
                if let Some(response) = &self.last_response {
                    Ok(response.clone())
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(feature = "dht2x")]
mod dht {
    pub(crate) fn humidity(data: &u16) -> f32 {
        *data as f32 / 10.0
    }

    pub(crate) fn temperature(data: &u16) -> f32 {
        let mut temperature = (data & 0x7FFF) as f32 / 10.0;
        if data & 0x8000 != 0 {
            temperature = -temperature;
        }
        temperature
    }
}

#[cfg(feature = "dht1x")]
mod dht {
    pub(crate) fn humidity(data: &u16) -> f32 {
        (*data >> 8) as f32 + ((*data & 0x00FF) as f32 * 0.1)
    }

    pub(crate) fn temperature(data: &u16) -> f32 {
        let mut temperature = ((data & 0x7FFF) >> 8) as f32 + ((*data & 0x00FF) as f32 * 0.1);
        if data & 0x8000 != 0 {
            temperature = -temperature;
        }
        temperature
    }
}

