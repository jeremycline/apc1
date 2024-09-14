//! Commands the host can send to the APC1.
//!
//! Each command is 7 bytes in length. The format of each command is:
//!
//! -----------------------------------------------
//! | 2 bytes      | 1 byte  | 2 bytes | 2 bytes  |
//! -----------------------------------------------
//! | Magic Number | Command |  Mode   | Checksum |
//! -----------------------------------------------
//!
//! The command is in big-endian byte order. The checksum applies to all bytes in the command
//! except the checksum itself.
//!
//! The device responds in the following format:
//!
//! ---------------------------------------------------------
//! | 2 bytes      | 2 bytes      | Variable     | 2 bytes  |
//! ---------------------------------------------------------
//! | Magic Number | Frame length | Mode or Data | Checksum |
//! ---------------------------------------------------------
//!
//! The response is also in big-endian byte order. The response size depends on the request.

/// The magic number that starts each command.
const MAGIC: &[u8; 2] = &[0x42, 0x4D];

// When this command is sent to the device with Mode Low set to 0x00 the
// device enters Idle mode. In this mode, the device fan is powered down,
// reducing device current from ~75mA to ~9mA.
//
// When Mode Low is set to 0x01 the device enters measurement mode by
// powering on the fan. This is the device's default mode on start-up.
const TOGGLE_DEVICE_MODE: u8 = 0xE4;
const MEASUREMENT_MODE: u8 = 1;
const IDLE_MODE: u8 = 0;

// Request the device's module type, ID, and firmware version.
const READ_MODULE_ID: u8 = 0xE9;

/// Refer to Section 8.3.3 of the datasheet.
///
/// Commands must be written to Write Register Address 0x40 - 0x46
/// Response in the format in [`Module`] is at address 0x47 - 0x5D
pub mod i2c {
    use super::{
        calculate_checksum, IDLE_MODE, MAGIC, MEASUREMENT_MODE, READ_MODULE_ID, TOGGLE_DEVICE_MODE,
    };

    /// The APC1-I's 7 bit I2C device address.
    pub const DEVICE_ADDR: u8 = 0x12;

    /// I2C clock frequency in Kbit/s.
    pub const SCLK_FREQUENCY: u8 = 100;

    /// The base Write Register Address commands should be written to.
    pub const COMMAND_BASE_ADDR: u8 = 0x40;

    // The I2C variant supports this option with the [`TOGGLE_DEVICE_MODE`] command.
    const RESET_DEVICE: u8 = 0x0F;

    /// Available commands for the I2C APC1.
    ///
    /// Commands should be written to Write Register Address 0x40 through 0x46.
    pub enum Command {
        /// Place the device into an idle state, which powers down the fan on the device,
        /// reducing device current from ~75mA to ~9mA.
        SetIdleMode,
        /// Place the device into an active state, which powers on the fan and consumes
        /// ~75mA. This is the default state of the device.
        SetActiveMode,
        /// Request the device restart and return to power-on defaults.
        Reset,
        /// Request the device's module type, ID, and firmware version.
        ReadModuleId,
    }

    impl Command {
        /// Convert the command to an array of bytes, suitable to be written to a UART device.
        pub fn to_bytes(&self) -> [u8; 7] {
            // It would be nice for this to be const at some point
            match self {
                Command::SetIdleMode => {
                    let mut command = [MAGIC[0], MAGIC[1], TOGGLE_DEVICE_MODE, 0, IDLE_MODE, 0, 0];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::SetActiveMode => {
                    let mut command = [
                        MAGIC[0],
                        MAGIC[1],
                        TOGGLE_DEVICE_MODE,
                        0,
                        MEASUREMENT_MODE,
                        0,
                        0,
                    ];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::ReadModuleId => {
                    let mut command = [MAGIC[0], MAGIC[1], READ_MODULE_ID, 0, 0, 0, 0];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::Reset => {
                    let mut command = [
                        MAGIC[0],
                        MAGIC[1],
                        TOGGLE_DEVICE_MODE,
                        0,
                        RESET_DEVICE,
                        0,
                        0,
                    ];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
            }
        }
    }
}

/// Commands for the UART variant of the APC1.
///
/// The APC1-U device operates with a baud rate of 9,600, 8 data bits, no
/// parity, and a stop bit of 1.
pub mod uart {
    use super::{
        calculate_checksum, IDLE_MODE, MAGIC, MEASUREMENT_MODE, READ_MODULE_ID, TOGGLE_DEVICE_MODE,
    };

    pub const BAUD_RATE: u16 = 9600;
    pub const DATA_BITS: u8 = 8;
    pub const STOP_BIT: u8 = 1;

    // When this command is sent to the device with Mode Low set to 0x00 the
    // device sends measurements on request. This is the default mode the
    // device starts in.
    //
    // When Mode Low is set to 0x01 the device enters active mode and sends a
    // measurement every second.
    const TOGGLE_MEASUREMENT_MODE: u8 = 0xE1;
    const ACTIVE_MEASUREMENT_MODE: u8 = 1;
    const PASSIVE_MEASUREMENT_MODE: u8 = 0;

    // Request the device send a measurement.
    const REQUEST_MEASUREMENT: u8 = 0xE2;

    /// Available commands for the UART APC1
    pub enum Command {
        /// When this command is sent to the device, it enters active mode and sends a
        /// measurement every second.
        SetActiveMeasurement,
        /// When this command is sent to the device, it sends measurements on
        /// request. This is the default mode the device starts in.
        SetPassiveMeasurement,

        /// When the device is in passive measurement mode, this command can be
        /// sent to request a new measurement from the device.
        RequestMeasurement,
        /// Place the device into an idle state, which powers down the fan on the device,
        /// reducing device current from ~75mA to ~9mA.
        SetIdleMode,
        /// Place the device into an active state, which powers on the fan and consumes
        /// ~75mA. This is the default state of the device.
        SetActiveMode,
        /// Request the device's module type, ID, and firmware version.
        ReadModuleId,
    }

    impl Command {
        /// Convert the command to an array of bytes, suitable to be written to a UART device.
        pub fn to_bytes(&self) -> [u8; 7] {
            // It would be nice for this to be const at some point
            match self {
                Command::SetActiveMeasurement => {
                    let mut command = [
                        MAGIC[0],
                        MAGIC[1],
                        TOGGLE_MEASUREMENT_MODE,
                        0,
                        ACTIVE_MEASUREMENT_MODE,
                        0,
                        0,
                    ];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::SetPassiveMeasurement => {
                    let mut command = [
                        MAGIC[0],
                        MAGIC[1],
                        TOGGLE_MEASUREMENT_MODE,
                        0,
                        PASSIVE_MEASUREMENT_MODE,
                        0,
                        0,
                    ];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::RequestMeasurement => {
                    let mut command = [MAGIC[0], MAGIC[1], REQUEST_MEASUREMENT, 0, 0, 0, 0];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::SetIdleMode => {
                    let mut command = [MAGIC[0], MAGIC[1], TOGGLE_DEVICE_MODE, 0, IDLE_MODE, 0, 0];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::SetActiveMode => {
                    let mut command = [
                        MAGIC[0],
                        MAGIC[1],
                        TOGGLE_DEVICE_MODE,
                        0,
                        MEASUREMENT_MODE,
                        0,
                        0,
                    ];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
                Command::ReadModuleId => {
                    let mut command = [MAGIC[0], MAGIC[1], READ_MODULE_ID, 0, 0, 0, 0];
                    let (payload, checksum) = command.split_at_mut(5);
                    checksum.copy_from_slice(calculate_checksum(payload).as_slice());

                    command
                }
            }
        }
    }
}

fn calculate_checksum(payload: &[u8]) -> [u8; 2] {
    let checksum: u16 = payload
        .iter()
        .fold(0_u16, |checksum, elem| checksum.wrapping_add(*elem as u16));

    checksum.to_be_bytes()
}
