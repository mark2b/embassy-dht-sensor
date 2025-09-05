#![no_std]
#![no_main]

#[cfg(not(any(feature = "dht1x", feature = "dht2x")))]
compile_error!("You must select a DHT sensor model with a feature flag: dht1x or dht2x");

#[cfg(all(feature = "dht1x", feature = "dht2x"))]
compile_error!("You must select only one DHT sensor model with a feature flag: dht1x or dht2x");


#[cfg(all(feature = "rp2040", not(feature = "rp_pio")))]
mod dht_rp;

#[cfg(all(feature = "rp2040", not(feature = "rp_pio")))]
pub use dht_rp::DHTSensor;

#[cfg(all(feature = "rp2040", feature = "rp_pio"))]
mod dht_rp_pio;
#[cfg(all(feature = "rp2040", feature = "rp_pio"))]
pub use dht_rp_pio::DHTSensor;

#[derive(Clone)]
pub struct DTHResponse {
    pub humidity: f32,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub enum DHTSensorError {
    Timeout,
    ChecksumError,
    InvalidData,
}

#[cfg(feature = "dht2x")]
mod dht {
    pub(crate) const MIN_REQUEST_INTERVAL_SECS: u64 = 2;
    pub(crate) const START_LOW_INTERVAL_US: u64 = 1_100; // 1ms
}

#[cfg(feature = "dht1x")]
mod dht {
    pub(crate) const MIN_REQUEST_INTERVAL_SECS: u64 = 1;
    pub(crate) const START_LOW_INTERVAL_US: u64 = 18_000; // 18ms
}

