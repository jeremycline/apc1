#![no_std]

use apc1_core::{i2c as apc1_i2c, Error, Measurement};
use embedded_hal::i2c::I2c;

pub struct Apc1Sensor<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Apc1Sensor<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn measurement(&mut self) -> Result<Measurement, Error> {
        let mut buf: [u8; 64] = [0; 64];
        self.i2c.read(apc1_i2c::DEVICE_ADDR, &mut buf).map_err(|_| Error::I2C)?;
        let measurement = Measurement::try_from(&buf)?;
        Ok(measurement)
    }
}
