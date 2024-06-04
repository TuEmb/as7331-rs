#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use as7331_rs::As7331;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*,
};

use log::{error, info};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let i2c0 = I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        400.kHz(),
        &clocks,
        None,
    );
    let lsb_a = 304.69 / ((1 << (11 - 8)) as f32) / ((1 << 9) as f32 / 1024.0) / 1000.0;
    let lsb_b = 398.44 / ((1 << (11 - 8)) as f32) / ((1 << 9) as f32 / 1024.0) / 1000.0;
    let lsb_c = 191.41 / ((1 << (11 - 8)) as f32) / ((1 << 9) as f32 / 1024.0) / 1000.0;
    let mut as7331_sensor = As7331::new(i2c0, delay, 0x74);

    let _ = as7331_sensor.power_up();
    let _ = as7331_sensor.reset();
    delay.delay_millis(100);
    let chip_id = as7331_sensor.get_chip_id().unwrap();

    if chip_id == 0x21 {
        let _ = as7331_sensor.set_configuration_mode();
        let _ = as7331_sensor.init(0, 0, 0x01, 40, 8, 9);
        delay.delay_millis(100);
        let _ = as7331_sensor.set_measurement_mode();
    } else {
        error!("Wrong chip id: {}", chip_id);
    }

    loop {
        let status = as7331_sensor.get_status().unwrap();
        if (status & 0x0008) != 0 {
            let all_data = as7331_sensor.read_all_data().unwrap();
            let temp = all_data[0];
            let uv_a = all_data[1];
            let uv_b = all_data[2];
            let uv_c = all_data[3];

            info!("AS7331 UV DATA:");
            info!("AS7331 UVA: {:.2} (uW/cm^2)", uv_a as f32 * lsb_a);
            info!("AS7331 UVB: {:.2} (uW/cm^2)", uv_b as f32 * lsb_b);
            info!("AS7331 UVC: {:.2} (uW/cm^2)", uv_c as f32 * lsb_c);
            info!(
                "AS7331 Temperature: {:.2} (Celcius)",
                temp as f32 * 0.05 - 66.9
            );
        }
    }
}
