/// Operations available for the APC1.
///
/// The general format for requests from the controller to the device is:
///
/// ---------------------------------------------------------------------------------------------
/// |  Start bytes (0x42 0x4D)  | Command | Mode High | Mode Low | Checksum High | Checksum Low |
/// ---------------------------------------------------------------------------------------------
///
/// The APC1 is expected to answer within 200 milliseconds.
///
/// The general format of responses from the device to the controller is:
/// ----------------------------------------------------------------------------------------------------------------------
/// |  Start bytes (0x42 0x4D)  | FrameLengthH | FrameLengthL | Command/Data | ModeL/Data | Checksum High | Checksum Low |
/// ----------------------------------------------------------------------------------------------------------------------
///
/// Checksum is the sum of the values of all bytes sent, excluding the checksum itself.
///
/// All values are big endian.
use std::fmt::Display;

/// Measurement data from the APC1.
///
/// This is based on the structure documented in Section 8.2.1 of the APC1
/// Datasheet. Each measurement is 64 bytes and is updated every second when
/// the device is in Measurement mode.
#[derive(Debug, PartialEq, Eq)]
pub struct Measurement {
    /// PM1.0 mass concentration in ug/m3; range 0-500.
    pub pm1_0: u16,
    /// PM2.5 mass concentration in ug/m3; range 0-1,000.
    pub pm2_5: u16,
    /// PM10 mass concentration in ug/m3; range 0-1,500.
    pub pm10: u16,
    /// PM1.0 mass concentration in atmospheric environment in ug/m3; range 0-500.
    pub pm1_0_in_air: u16,
    /// PM2.5 mass concentration in atmospheric environment in ug/m3; range 0-1,000.
    pub pm2_5_in_air: u16,
    /// PM10 mass concentration in atmospheric environment in ug/m3; range 0-1,500.
    pub pm10_in_air: u16,
    /// Number of particles with diameter >0.3 micrometers in 0.1 liters of air; range 0-65535.
    pub um_0_3_particles: u16,
    /// Number of particles with diameter >0.5 micrometers in 0.1 liters of air; range 0-65535.
    pub um_0_5_particles: u16,
    /// Number of particles with diameter >1.0 micrometers in 0.1 liters of air; range 0-65535.
    pub um_1_particles: u16,
    /// Number of particles with diameter >2.5 micrometers in 0.1 liters of air; range 0-65535.
    pub um_2_5_particles: u16,
    /// Number of particles with diameter >5.0 micrometers in 0.1 liters of air; range 0-65535.
    pub um_5_particles: u16,
    /// Number of particles with diameter >10.0 micrometers in 0.1 liters of air; range 0-65535.
    pub um_10_particles: u16,
    /// TVOC output in ppb; range 0-65,000.
    pub tvoc: u16,
    /// CO2 equivalents in ppm, range 400-65,000
    pub eco2: u16,
    /// Reserved field.
    _reserved: u16,
    /// Temperature compensation in units of 0.1C; range 0-500 (0-50C).
    /// Compensation only valid for the module when the inlet and outlet is
    /// facing downwards (orientation 4 - see Section 9.2 of the datasheet).
    pub t_comp: u16,
    /// Relative humidity (RH) compensation in units of 0.1% RH; range 0-1000
    /// (0-100% RH). Compensation only valid for the module when the inlet and
    /// outlet is facing downwards (orientation 4 - see Section 9.2 of the
    /// datasheet).
    pub rh_comp: u16,
    /// Uncompensated temperature measurement in units of 0.1C; range 0-500 (0-50C).
    pub t_raw: u16,
    /// Uncompensated relative humidity (RH) in units of 0.1% RH; range 0-1000.
    pub rh_raw: u16,
    /// Gas sensor raw resistance value RS0; range 100-50M mapped to 1k-10M Ohms.
    pub rs_0: u32,
    /// Unused raw resistence value.
    pub rs_1: u32,
    /// Gas sensor raw resistance value for RS2; range 100-50M mapped to 1k-10M Ohms.
    pub rs_2: u32,
    /// Gas sensor raw resistance value for RS3; range 100-50M mapped to 1k-10M Ohms.
    pub rs_3: u32,
    /// The Air Quality Index according to the UBA Classification of TVOC value; range 1-5.
    pub aqi: u8,
    /// Reserved field.
    __reserved: u8,
    /// Device Firmware version.
    pub version: u8,
}

impl TryFrom<&[u8; 64]> for Measurement {
    type Error = crate::Error;

    fn try_from(value: &[u8; 64]) -> Result<Self, Self::Error> {
        // The final two bytes are a checksum of the first 62; validate that before continuing.
        let expected_checksum: u16 = u16::from_be_bytes([value[62], value[63]]);
        let actual_checksum: u16 = value
            .iter()
            .take(62)
            .fold(0_u16, |acc, elem| acc.wrapping_add(*elem as u16));
        if expected_checksum != actual_checksum {
            return Err(Self::Error::Checksum {
                expected: expected_checksum,
                actual: actual_checksum,
            });
        }

        // The frame header, expected to be 0x42 0x4D, followed by the frame length
        // which is everything after the frame header and the length field itself.
        // It should always be 60.
        if value[..4] != [0x42, 0x4D, 0x00, 0x3C] {
            return Err(Self::Error::InvalidHeader);
        }

        if value[61] != 0x00 {
            return Err(Self::Error::Device(DeviceErrorCode(value[61])));
        }

        Ok(Self {
            pm1_0: u16::from_be_bytes([value[4], value[5]]),
            pm2_5: u16::from_be_bytes([value[6], value[7]]),
            pm10: u16::from_be_bytes([value[8], value[9]]),
            pm1_0_in_air: u16::from_be_bytes([value[10], value[11]]),
            pm2_5_in_air: u16::from_be_bytes([value[12], value[12]]),
            pm10_in_air: u16::from_be_bytes([value[14], value[15]]),
            um_0_3_particles: u16::from_be_bytes([value[16], value[17]]),
            um_0_5_particles: u16::from_be_bytes([value[18], value[19]]),
            um_1_particles: u16::from_be_bytes([value[20], value[21]]),
            um_2_5_particles: u16::from_be_bytes([value[22], value[23]]),
            um_5_particles: u16::from_be_bytes([value[24], value[25]]),
            um_10_particles: u16::from_be_bytes([value[26], value[27]]),
            tvoc: u16::from_be_bytes([value[28], value[29]]),
            eco2: u16::from_be_bytes([value[30], value[31]]),
            _reserved: u16::from_be_bytes([value[32], value[33]]),
            t_comp: u16::from_be_bytes([value[34], value[35]]),
            rh_comp: u16::from_be_bytes([value[36], value[37]]),
            t_raw: u16::from_be_bytes([value[38], value[39]]),
            rh_raw: u16::from_be_bytes([value[40], value[41]]),
            rs_0: u32::from_be_bytes([value[42], value[43], value[44], value[45]]),
            rs_1: u32::from_be_bytes([value[46], value[47], value[48], value[49]]),
            rs_2: u32::from_be_bytes([value[50], value[51], value[52], value[53]]),
            rs_3: u32::from_be_bytes([value[54], value[55], value[56], value[57]]),
            aqi: value[58],
            __reserved: value[59],
            version: value[60],
        })
    }
}

impl Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Measurement:\n\t",
                "Temperature: {}C\n\t",
                "Humidity: {}%\n\t",
                "AQI: {}\n\t",
                "eCO2: {}ppm\n\t",
                "TVOC: {}ppb\n\t",
                "PM1.0: {}ug/m3\n\t",
                "PM2.5: {}ug/m3\n\t",
                "PM10: {}ug/m3\n\t",
                "PM1.0 in air: {}ug/m3\n\t",
                "PM2.5 in air: {}ug/m3\n\t",
                "PM10 in air: {}ug/m3\n\t",
                "# particles >0.3μm per 0.1L of air: {}\n\t",
                "# particles >0.5μm per 0.1L of air: {}\n\t",
                "# particles >1.0μm per 0.1L of air: {}\n\t",
                "# particles >2.5μm per 0.1L of air: {}\n\t",
                "# particles >5.0μm per 0.1L of air: {}\n\t",
                "# particles >10.0μm per 0.1L of air: {}\n\t",
            ),
            self.t_comp as f32 / 10.0,
            self.rh_comp as f32 / 10.0,
            self.aqi,
            self.eco2,
            self.tvoc,
            self.pm1_0,
            self.pm2_5,
            self.pm10,
            self.pm1_0_in_air,
            self.pm2_5_in_air,
            self.pm10_in_air,
            self.um_0_3_particles,
            self.um_0_5_particles,
            self.um_1_particles,
            self.um_2_5_particles,
            self.um_5_particles,
            self.um_10_particles,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeviceErrorCode(u8);

// Displays device errors.
//
// 0 indicates no errors. Errors are defined in Section 8.2.2 of the datasheet.
//
// -------------------------------------------------------------------------------------------------------------------------
// | Bit 7  |        Bit 6         |    Bit 5   | Bit 4 |    Bit 3    |    Bit 2   |     Bit 1     |         Bit 0         |
// -------------------------------------------------------------------------------------------------------------------------
// | Unused | Temp/Humidity Sensor | VOC Sensor | Laser | Fan Stopped | Photodiode | Fan-speed low | Too many Fan restarts |
// -------------------------------------------------------------------------------------------------------------------------
impl Display for DeviceErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Too many fan restarts: {}\n",
                "Fan-speed low: {}\n",
                "Photodiode: {}\n",
                "Fan stopped: {}\n",
                "Laser: {}\n",
                "VOC sensor: {}\n",
                "Temperature or Humidity sensor: {}\n",
            ),
            self.0 ^ (1 << 0),
            self.0 ^ (1 << 1),
            self.0 ^ (1 << 2),
            self.0 ^ (1 << 3),
            self.0 ^ (1 << 4),
            self.0 ^ (1 << 5),
            self.0 ^ (1 << 6),
        )
    }
}

/// Read the module firmware and version.
#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    /// The module's name and type encoded as ASCII.
    pub name_and_type: String,
    /// Module serial number.
    pub serial_number: u64,
    /// The delimiter character in the name_and_type string between the name and type.
    pub delimiter: char,
    /// The module's firmware version.
    pub fw_version_major: u8,
    pub fw_version_minor: u8,
}

impl TryFrom<&[u8; 23]> for Module {
    type Error = crate::Error;

    fn try_from(value: &[u8; 23]) -> Result<Self, Self::Error> {
        let expected_checksum: u16 = u16::from_be_bytes([value[21], value[22]]);
        let actual_checksum: u16 = value
            .iter()
            .take(21)
            .fold(0_u16, |acc, elem| acc.wrapping_add(*elem as u16));
        if expected_checksum != actual_checksum {
            return Err(Self::Error::Checksum {
                expected: expected_checksum,
                actual: actual_checksum,
            });
        }
        // The frame header, expected to be 0x42 0x4D, followed by the frame length
        // which is everything after the frame header and the length field itself.
        if value[..4] != [0x42, 0x4D, 0x00, 0x13] {
            return Err(Self::Error::InvalidHeader);
        }

        Ok(Self {
            name_and_type: value.iter().skip(4).take(6).map(|b| *b as char).collect(),
            serial_number: u64::from_be_bytes([
                value[10], value[11], value[12], value[13], value[14], value[15], value[16],
                value[17],
            ]),
            delimiter: value[18] as char,
            fw_version_major: value[19],
            fw_version_minor: value[20],
        })
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Name and type: {}\n",
                "Serial number: {}\n",
                "Name/type delimiter: {}\n",
                "Firmware version: {}.{}\n",
            ),
            self.name_and_type,
            self.serial_number,
            self.delimiter,
            self.fw_version_major,
            self.fw_version_minor,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Error;

    #[test]
    fn try_from_valid_measurement() {
        let valid_measurement: &[u8; 64] = &[
            66, 77, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 100, 0, 20, 0, 10, 0, 0,
            0, 0, 0, 37, 1, 173, 0, 1, 0, 202, 2, 69, 0, 251, 1, 181, 0, 3, 101, 146, 0, 0, 0, 1,
            0, 12, 19, 208, 0, 0, 147, 102, 1, 0, 35, 0, 8, 59,
        ];

        let measurement: Measurement = valid_measurement.try_into().unwrap();
        assert_eq!(measurement.aqi, 1);
    }

    #[test]
    fn try_from_measurement_invalid_checksum() {
        // Same measurement as above with one byte incremented by 1.
        let invalid_measurement: &[u8; 64] = &[
            66, 77, 0, 60, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 80, 0, 100, 0, 20, 0, 10, 0, 0,
            0, 0, 0, 37, 1, 173, 0, 1, 0, 202, 2, 69, 0, 251, 1, 181, 0, 3, 101, 146, 0, 0, 0, 1,
            0, 12, 19, 208, 0, 0, 147, 102, 1, 0, 35, 0, 8, 59,
        ];
        let actual = Measurement::try_from(invalid_measurement);
        let expected = Err(Error::Checksum {
            expected: 2107,
            actual: 2108,
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn try_from_measurement_invalid_header() {
        // Invalid header with a miraculously correct checksum
        let invalid_measurement: &[u8; 64] = &[
            77, 66, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 100, 0, 20, 0, 10, 0, 0,
            0, 0, 0, 37, 1, 173, 0, 1, 0, 202, 2, 69, 0, 251, 1, 181, 0, 3, 101, 146, 0, 0, 0, 1,
            0, 12, 19, 208, 0, 0, 147, 102, 1, 0, 35, 0, 8, 22,
        ];
        let actual = Measurement::try_from(invalid_measurement);
        let expected = Err(Error::InvalidHeader);

        assert_eq!(actual, expected);
    }

    #[test]
    fn try_from_module_valid() {
        let valid_module: &[u8; 23] = &[
            66, 77, 0, 19, 65, 80, 67, 49, 45, 73, 8, 110, 169, 135, 242, 77, 68, 134, 45, 0, 35,
            6, 28,
        ];
        let actual: Module = valid_module.try_into().unwrap();
        assert_eq!(actual.name_and_type, "APC1-I");
        assert_eq!(actual.serial_number, 607609401092424838_u64);
        assert_eq!(actual.delimiter, '-');
        assert_eq!(actual.fw_version_major, 0);
        assert_eq!(actual.fw_version_minor, 35);
    }

    #[test]
    fn try_from_module_invalid_checksum() {
        // Same measurement as above with one byte incremented by 1.
        let invalid_module: &[u8; 23] = &[
            66, 77, 0, 19, 65, 80, 67, 49, 46, 73, 8, 110, 169, 135, 242, 77, 68, 134, 45, 0, 35,
            6, 28,
        ];
        let actual = Module::try_from(invalid_module);
        let expected = Err(Error::Checksum {
            expected: 1564,
            actual: 1565,
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn try_from_module_invalid_header() {
        // Correct checksum, incorrect header
        let invalid_module: &[u8; 23] = &[
            67, 77, 0, 19, 65, 80, 67, 49, 46, 73, 8, 110, 169, 135, 242, 77, 68, 134, 45, 0, 35,
            6, 30,
        ];
        let actual = Module::try_from(invalid_module);
        let expected = Err(Error::InvalidHeader);

        assert_eq!(actual, expected);
    }
}
