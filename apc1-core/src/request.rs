/// Commands the host can send to the APC1.

pub mod i2c {

    /// The APC1-I's device address.
    pub const DEVICE_ADDR: u16 = 0x12;

    /// Refer to Section 8.3.3 of the datasheet.
    ///
    /// Commands must be written to Write Register Address 0x40 - 0x46
    /// Response in the format in [`Module`] is at address 0x47 - 0x5D
    ///
    /// Format is:
    /// ---------------------------------------------------------------------------------------------
    /// |  Start bytes (0x42 0x4D)  | Command | Mode High | Mode Low | Checksum High | Checksum Low |
    /// ---------------------------------------------------------------------------------------------
    ///
    /// Checksum is the sum of the values of all bytes sent, excluding the checksum itself, with
    /// Checksum H = High byte
    /// Checksum L = Low byte
    pub const READ_MODULE: [u8; 7] = [0x42, 0x4D, 0xE9, 0x00, 0x00, 0x01, 0x78];
    pub const SET_MEASURMENT_MODE: [u8; 7] = [0x42, 0x4d, 0xe4, 0x00, 0x0F, 0x01, 0x74];
}

/// The APC1-U device operates with a baud rate of 9,600, 8 data bits, no
/// parity, and a stop bit of 1.
pub mod uart {
    pub const BAUD_RATE: u16 = 9600;
    pub const DATA_BITS: u8 = 8;
    pub const STOP_BIT: u8 = 1;

    /// When this command is sent to the device with Mode Low set to 0x00 the
    /// device sends measurements on request. This is the default mode the
    /// device starts in.
    ///
    /// When Mode Low is set to 0x01 the device enters active mode and sends a
    /// measurement every second.
    pub const TOGGLE_MEASUREMENT_MODE: u16 = 0xE1;

    /// When the device is in passive measurement mode (see
    /// [`TOGGLE_MEASUREMENT_MODE`]), this command can be sent to request a new
    /// measurement from the device.
    pub const REQUEST_MEASUREMENT: u16 = 0xE2;

    /// When this command is sent to the device with Mode Low set to 0x00 the
    /// device enters Idle mode. In this mode, the device fan is powered down,
    /// reducing device current from ~75mA to ~9mA.
    ///
    /// When Mode Low is set to 0x01 the device enters measurement mode by
    /// powering on the fan. This is the device's default mode on start-up.
    pub const TOGGLE_DEVICE_MODE: u16 = 0xE4;

    /// Request the device's module type, ID, and firmware version.
    pub const READ_MODULE_ID: u16 = 0xE9;
}
