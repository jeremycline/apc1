#![no_std]

mod request;
mod response;

pub use request::{i2c, uart};
pub use response::{DeviceErrorCode, Measurement, Module};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    InvalidHeader,
    InvalidName,
    Checksum {
        expected: u16,
        actual: u16,
    },
    Device(DeviceErrorCode),
    #[cfg(feature = "embedded")]
    I2C,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidHeader => write!(f, "Reading was missing the expected frame header"),
            Error::InvalidName => write!(f, "Device name was not valid ASCII"),
            Error::Checksum { expected, actual } => {
                write!(f, "Checksum failed: expected {expected:x}, got {actual:x}")
            }
            Error::Device(state) => write!(f, "Device in error state: {}", state),
            #[cfg(feature = "embedded")]
            Error::I2C => write!(f, "I2C error"),
        }
    }
}
