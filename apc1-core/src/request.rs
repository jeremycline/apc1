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
