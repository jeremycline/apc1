mod request;
mod response;

pub use request::{uart, i2c};
pub use response::{DeviceErrorCode, Measurement, Module};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Reading was missing the expected frame header")]
    InvalidHeader,
    #[error("Checksum failed: expected {expected:?}, got {actual:?}")]
    Checksum { expected: u16, actual: u16 },
    #[error("Device in error state: {0}")]
    Device(DeviceErrorCode),
}
