use i2cdev::core::*;
use i2cdev::linux;
use std::error::Error;
use crate::sensors::Sensor;

#[repr(u8)]
enum VCNL4040Address {
    AlsData = 0x09,
}

pub struct VCNL4040Sensor {
    device: linux::LinuxI2CDevice,
}

impl VCNL4040Sensor {
    pub fn new(device_path: &str, address: u16) -> Result<VCNL4040Sensor, linux::LinuxI2CError> {
        let device = linux::LinuxI2CDevice::new(device_path, address)?;
        Ok(Self { device })
    }

    pub fn set_als_config(&mut self, cmd: u8) -> Result<(), linux::LinuxI2CError> {
        self.write_register(0x00, cmd, 0x00)
    }

    fn read_register(&mut self, reg_addr: u8) -> Result<u16, linux::LinuxI2CError> {
        // TODO: なぜこの方法ではうまくいかないか調べる
        // self.device.write(&reg_addr.to_be_bytes())?;
        // self.device.read(&mut buffer)?;
        let buffer = self.device.smbus_read_word_data(reg_addr)?;
        Ok(buffer)
    }

    fn write_register(&mut self, reg_addr: u8, lsb: u8, msb: u8) -> Result<(), linux::LinuxI2CError> {
        let buffer = [reg_addr, lsb, msb];
        self.device.write(&buffer)?;
        println!("Wrote to register 0x{:02x}:", reg_addr);
        Ok(())
    }
}

impl Sensor for VCNL4040Sensor {
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>> {
        let value = self.read_register(VCNL4040Address::AlsData as u8)? as f32 * 0.1;
        // println!("Ambient Light: {}", value);
        Ok(value as u16)
    }
}
