use i2cdev::core::I2CDevice;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use i2cdev::linux::LinuxI2CDevice;

use apc1_core::request::i2c;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Full path to the i2c device provided by the i2c-dev kernel module.
    /// For example: /dev/i2c-1
    #[arg(short, long)]
    i2c_device: PathBuf,
    #[command(subcommand)]
    request: Request,
}

#[derive(Subcommand, Debug)]
enum Request {
    /// Show the module's name, serial number, and firmware version
    Module,
    /// Read air quality measurements from the device.
    Measurement,
}

fn read_module(dev: &mut LinuxI2CDevice) -> anyhow::Result<apc1_core::Module> {
    let mut buf: [u8; 23] = [0; 23];
    let mut base_addr = 0x40_u8;
    for b in i2c::READ_MODULE.iter() {
        dev.smbus_write_byte_data(base_addr, *b)
            .with_context(|| "Failed to write the readmodule command")?;
        base_addr += 1;
    }
    let base_addr = 0x47_u8;
    for i in 0..23_u8 {
        buf[i as usize] = dev
            .smbus_read_byte_data(base_addr + i)
            .with_context(|| "Failed to read response into buffer")?;
    }
    apc1_core::Module::try_from(&buf).with_context(|| "Response was invalid")
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut dev = LinuxI2CDevice::new(args.i2c_device, i2c::DEVICE_ADDR)
        .with_context(|| "Unable to open the I2C device file. Is the i2c-dev module loaded?")?;

    match args.request {
        Request::Module => {
            let mut read_tries = 0;
            loop {
                if read_tries > 10 {
                    anyhow::bail!("Unable to detect module on the provided I2C device.");
                }
                if let Ok(module) = read_module(&mut dev) {
                    println!("{}", module);
                    break;
                }
                read_tries += 1;
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        }
        Request::Measurement => {
            let command = vec![0x42, 0x4d, 0xe4, 0x00, 0x0F, 0x01, 0x74];
            let mut base_addr = 0x40_u8;
            for b in command.into_iter() {
                dev.smbus_write_byte_data(base_addr, b)
                    .with_context(|| "Failed to write the Measurement Mode command")?;
                base_addr += 1;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));

            let mut buf: [u8; 64] = [0; 64];
            for _ in 0..300 {
                dev.read(&mut buf)
                    .with_context(|| "Failed to read response into buffer")?;
                if let Ok(measurement) = apc1_core::Measurement::try_from(&buf) {
                    println!("{}", measurement);
                }
                std::thread::sleep(std::time::Duration::from_millis(1500));
            }
        }
    }

    Ok(())
}
