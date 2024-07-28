use std::{num::NonZeroU64, path::PathBuf};

use anyhow::Context;
use apc1_core::i2c;
use apc1_core::Measurement;
use clap::{Parser, Subcommand};
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use time::OffsetDateTime;
use tokio::sync::mpsc::{self, Receiver};

static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/");

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
    /// Read current air quality measurements from the device and print it to stdout
    Measurement,
    /// Log measurements to a PostgreSQL database
    Log {
        /// The database URI
        #[arg(env = "APC1_DB_URI")]
        db_uri: String,
        /// How frequently (in seconds) to record a measurement.
        #[arg(long)]
        interval: NonZeroU64,
        /// Identifies where the device is.
        #[arg(long)]
        location: String,
    },
}

fn read_sensor(
    mut dev: LinuxI2CDevice,
    interval: u64,
    dest: mpsc::Sender<(OffsetDateTime, Measurement)>,
) -> anyhow::Result<()> {
    let interval = std::time::Duration::from_secs(interval);
    loop {
        let mut buf: [u8; 64] = [0; 64];
        dev.read(&mut buf)
            .with_context(|| "Failed to read response into buffer")?;
        match Measurement::try_from(&buf) {
            Ok(measurement) => {
                tracing::debug!("Read measurement successfully");
                let measurement_time = OffsetDateTime::now_utc();
                dest.blocking_send((measurement_time, measurement))?;
            }
            Err(e) => {
                tracing::warn!(error=?e, "Measurement reading was invalid");
                std::thread::sleep(std::time::Duration::from_millis(1100));
                continue;
            }
        }
        std::thread::sleep(interval);
    }
}

fn read_module(dev: &mut LinuxI2CDevice) -> anyhow::Result<apc1_core::Module> {
    for (index, byte) in i2c::Command::ReadModuleId.to_bytes().iter().enumerate() {
        dev.smbus_write_byte_data(i2c::COMMAND_BASE_ADDR + index as u8, *byte)
            .with_context(|| "Failed to write the readmodule command")?;
    }
    let mut buf: [u8; 23] = [0; 23];
    let base_addr = 0x47_u8;
    for i in 0..23_u8 {
        buf[i as usize] = dev
            .smbus_read_byte_data(base_addr + i)
            .with_context(|| "Failed to read response into buffer")?;
    }
    apc1_core::Module::try_from(&buf).with_context(|| "Response was invalid")
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut dev = LinuxI2CDevice::new(args.i2c_device, i2c::DEVICE_ADDR.into())
        .with_context(|| "Unable to open the I2C device file. Is the i2c-dev module loaded?")?;

    // Ensure the device is in a known state by requesting it reset.
    for (index, byte) in i2c::Command::Reset.to_bytes().iter().enumerate() {
        dev.smbus_write_byte_data(i2c::COMMAND_BASE_ADDR + index as u8, *byte)
            .with_context(|| "Failed to write the readmodule command")?;
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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
        Request::Log {
            db_uri,
            interval,
            location,
        } => {
            tracing_subscriber::fmt::init();
            let pool = PgPoolOptions::new()
                .max_connections(3)
                .connect(&db_uri)
                .await?;
            MIGRATIONS.run(&pool).await?;

            let device = loop {
                match read_module(&mut dev) {
                    Ok(module) => break module,
                    Err(e) => {
                        tracing::warn!(error=?e, "Failed to read I2C device; trying again...")
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            };
            tracing::info!(
                name = device.name_and_type,
                serial_number = device.serial_number,
                "Detected APC1 sensor"
            );

            let (sender, receiver) = mpsc::channel(64);
            let db_writer = tokio::spawn(write_results(
                location,
                device.serial_number.to_string(),
                pool,
                receiver,
            ));
            let sensor_reader = tokio::task::spawn_blocking(move || {
                read_sensor(dev, interval.into(), sender).unwrap();
            });
            let _result = tokio::join!(db_writer, sensor_reader);
        }
    }

    Ok(())
}

async fn write_results(
    location: String,
    device_id: String,
    db: Pool<Postgres>,
    mut receiver: Receiver<(OffsetDateTime, Measurement)>,
) -> anyhow::Result<()> {
    while let Some((measurement_time, measurement)) = receiver.recv().await {
        if let Err(e) = sqlx::query!(
                "
                INSERT INTO apc_reading (
                    measurement_time,
                    location,
                    device_sn,
                    tvoc,
                    eco2,
                    aqi,
                    temperature,
                    humidity,
                    pm1_0,
                    pm2_5,
                    pm10,
                    pm1_0_in_air,
                    pm2_5_in_air,
                    pm10_in_air,
                    um0_3_particles,
                    um0_5_particles,
                    um1_particles,
                    um2_5_particles,
                    um5_particles,
                    um10_particles
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                ",
                measurement_time,
                &location,
                &device_id,
                measurement.tvoc as i32,
                measurement.eco2 as i32,
                measurement.aqi as i32,
                measurement.t_comp as i32,
                measurement.rh_comp as i32,
                measurement.pm1_0 as i32,
                measurement.pm2_5 as i32,
                measurement.pm10 as i32,
                measurement.pm1_0_in_air as i32,
                measurement.pm2_5_in_air as i32,
                measurement.pm10_in_air as i32,
                measurement.um_0_3_particles as i32,
                measurement.um_0_5_particles as i32,
                measurement.um_1_particles as i32,
                measurement.um_2_5_particles as i32,
                measurement.um_5_particles as i32,
                measurement.um_10_particles as i32,
            ).execute(&db).await {
                tracing::error!(error=?e, "Failed to write measurement to database");
            } else {
                tracing::info!(location, device_id, "Logged measurement successfully");
            }
    }
    Ok(())
}
