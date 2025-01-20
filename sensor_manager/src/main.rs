use std::thread;
use std::error::Error;
use gpiod;
use i2cdev::core::*;
use i2cdev::linux;

fn main() {
    println!("Hello, world!");
}

trait Sensor {
    fn collect_data() -> Result<u16, Box<dyn Error>>; // TODO: なぜヒープに格納するのか
}

struct VCNL4004Sensor {
    device: linux::LinuxI2CDevice,
}

impl VCNL4004Sensor {
    fn new(device_path: &str, address: u16) -> Result<Self, linux::LinuxI2CError> {
        let device = linux::LinuxI2CDevice::new(device_path, address)?;
        Ok(Self { device });
    }
    fn readRegister(&mut self, reg_addr: u16) -> Result<u16, linux::LinuxI2CError> { 
        let mut buffer[0u8, 2];

        // レジスタアドレスを送信
        self.device.write(&[reg_addr]); // TODO: 構文を確認

        // データを受信
        self.device.read(&mut buffer);

        // 16ビット値として返す
        let data = u16::from_be_bytes(buffer); // TODO: 関数の意味
        Ok(data)
    }
}

impl Sensor for VCNL4004Sensor {
    fn collect_data() {
        let value = self.readRegister(0x09); // TODO: 定数にする
        let value = value * 0.1;
        println!("Ambient Light: {}", data);
        Ok(value as u16)
    }
}

struct HCSR5015Sensor {
    chip: gpiod::Chip,
    line: gpiod::Line
}

impl Sensor for HCSR5015Sensor {
    fn collect_data() {
        let value = self.line.get_value()?;
        Ok(value as u16)
    }
}