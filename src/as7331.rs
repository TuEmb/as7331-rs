use core::result::Result::{self, Err, Ok};
use esp_hal::i2c::Error;
use esp_hal::peripherals::I2C0;
use esp_hal::Blocking;
use esp_hal::{delay::Delay, i2c::I2C};
use log::debug;

// Configuration State Registers
const AS7331_OSR: u8 = 0x00;
const AS7331_AGEN: u8 = 0x02;
const AS7331_CREG1: u8 = 0x06;
const AS7331_CREG3: u8 = 0x08;
const AS7331_BREAK: u8 = 0x09;

// Measurement State registers
const AS7331_STATUS: u8 = 0x00;
const AS7331_TEMP: u8 = 0x01;
const AS7331_MRES1: u8 = 0x02;
const AS7331_MRES2: u8 = 0x03;
const AS7331_MRES3: u8 = 0x04;

pub struct As7331<'a> {
    pub i2c: I2C<'a, I2C0, Blocking>,
    pub addr: u8,
    pub delay: Delay,
}

#[allow(dead_code)]
impl<'a> As7331<'a> {
    pub fn new(i2c: I2C<'a, I2C0, Blocking>, delay: Delay, addr: u8) -> Self {
        As7331 { i2c, delay, addr }
    }

    pub fn destroy(self) -> I2C<'a, I2C0, Blocking> {
        self.i2c
    }

    pub fn get_chip_id(&mut self) -> Result<u8, Error> {
        let mut data = [0u8; 1];
        self.i2c_write_read_cmd(AS7331_AGEN, &mut data)?;
        Ok(data[0])
    }

    pub fn init(
        &mut self,
        mmode: u8,
        cclk: u8,
        sb: u8,
        break_time: u8,
        gain: u8,
        time: u8,
    ) -> Result<(), Error> {
        self.i2c_write_cmd(AS7331_CREG1, gain << 4 | time)?;
        self.i2c_write_cmd(AS7331_CREG3, mmode << 6 | sb << 4 | cclk)?;
        self.i2c_write_cmd(AS7331_BREAK, break_time)
    }

    pub fn one_shot(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 1];
        self.i2c_write_read_cmd(AS7331_OSR, &mut data)?;
        self.i2c_write_cmd(AS7331_OSR, data[0] | 0x80)
    }

    pub fn get_status(&mut self) -> Result<u16, Error> {
        let mut data = [0u8; 2];
        self.i2c_read_bytes(AS7331_STATUS, &mut data)?;
        Ok(((data[1] as u16) << 8) | (data[0] as u16))
    }

    pub fn read_temp_data(&mut self) -> Result<u16, Error> {
        let mut data = [0u8; 2];
        self.i2c_read_bytes(AS7331_TEMP, &mut data)?;
        Ok(((data[1] as u16) << 8) | (data[0] as u16))
    }

    pub fn read_uv_a_data(&mut self) -> Result<u16, Error> {
        let mut data = [0u8; 2];
        self.i2c_read_bytes(AS7331_MRES1, &mut data)?;
        Ok(((data[1] as u16) << 8) | (data[0] as u16))
    }

    pub fn read_uv_b_data(&mut self) -> Result<u16, Error> {
        let mut data = [0u8; 2];
        self.i2c_read_bytes(AS7331_MRES2, &mut data)?;
        Ok(((data[1] as u16) << 8) | (data[0] as u16))
    }

    pub fn read_uv_c_data(&mut self) -> Result<u16, Error> {
        let mut data = [0u8; 2];
        self.i2c_read_bytes(AS7331_MRES3, &mut data)?;
        Ok(((data[1] as u16) << 8) | (data[0] as u16))
    }

    pub fn read_all_data(&mut self) -> Result<[u16; 4], Error> {
        let mut raw_data = [0u8; 8];
        self.i2c_read_bytes(AS7331_TEMP, &mut raw_data)?;
        Ok([
            ((raw_data[1] as u16) << 8) | (raw_data[0] as u16),
            ((raw_data[3] as u16) << 8) | (raw_data[2] as u16),
            ((raw_data[5] as u16) << 8) | (raw_data[4] as u16),
            ((raw_data[7] as u16) << 8) | (raw_data[6] as u16),
        ])
    }

    fn i2c_write_read_cmd(&mut self, addr: u8, data: &mut [u8]) -> Result<(), Error> {
        match self.i2c.write_read(self.addr, &[addr], data) {
            Ok(_) => debug!(
                "I2C_WRITE_READ - ADDR: 0x{:02X} - READ: 0x{:02X}",
                addr, data[0]
            ),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn i2c_read_bytes(&mut self, addr: u8, data: &mut [u8]) -> Result<(), Error> {
        match self.i2c.write_read(self.addr, &[addr], data) {
            Ok(_) => debug!("I2C_READ_BYTES - ADDR: 0x{:02X} - DATA {:?}", addr, data),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn i2c_write_cmd(&mut self, addr: u8, cmd: u8) -> Result<(), Error> {
        match self.i2c.write(self.addr, &[addr, cmd]) {
            Ok(_) => debug!("I2C_WRITE - ADDR: 0x{:02X} - DATa: 0x{:02X}", addr, cmd),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn power_up(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 22];
        self.i2c_write_read_cmd(AS7331_OSR, &mut data)?;
        self.i2c_write_cmd(AS7331_OSR, data[0] | 0x40)
    }

    pub fn power_down(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 22];
        self.i2c_write_read_cmd(AS7331_OSR, &mut data)?;

        self.i2c_write_cmd(AS7331_OSR, data[0] & !0x40)
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 22];
        match self.i2c_write_read_cmd(AS7331_OSR, &mut data) {
            Err(e) => return Err(e),
            _ => {}
        }

        self.i2c_write_cmd(AS7331_OSR, data[0] | 0x08)
    }

    pub fn set_configuration_mode(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 22];
        match self.i2c_write_read_cmd(AS7331_OSR, &mut data) {
            Err(e) => return Err(e),
            _ => {}
        }

        self.i2c_write_cmd(AS7331_OSR, data[0] | 0x02)
    }

    pub fn set_measurement_mode(&mut self) -> Result<(), Error> {
        let mut data = [0u8; 22];
        match self.i2c_write_read_cmd(AS7331_OSR, &mut data) {
            Err(e) => return Err(e),
            _ => {}
        }

        self.i2c_write_cmd(AS7331_OSR, data[0] | 0x83)
    }
}
