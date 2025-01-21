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


#[repr(u8)] 
enum VCNL4040Address {
    AlsConf = 0x00,
    AlsData = 0x09,
}

struct VCNL4004Sensor {
    device: linux::LinuxI2CDevice,
}

impl VCNL4004Sensor {
    fn new(device_path: &str, address: u16) -> Result<Self, linux::LinuxI2CError> {
        let device = linux::LinuxI2CDevice::new(device_path, address)?;
        Ok(Self { device });
    }

    fn setAlsConfig(cmd: u8) -> Result<(), linux::LinuxI2CError>{
        self.writeRegister(VCNL4040Address::AlsConf as u16, cmd, 0x00);
    }

    fn readRegister(&mut self, reg_addr: u16) -> Result<u16, linux::LinuxI2CError> { 
        let mut buffer = [0u8; 2];

        // レジスタアドレスを送信
        self.device.write(&reg_addr.to_be_bytes())?; // TODO: 構文を確認

        // データを受信
        self.device.read(&mut buffer)?;

        // 16ビット値として返す
        let data = u16::from_be_bytes(buffer); // TODO: 関数の意味
        Ok(data)impl Sensor for VCNL4004Sensor {
            fn collect_data() {
                let value = self.readRegister(VCNL4040Address::AlsData);
                let value = value * 0.1; // TODO: luxの計算があっているか確認
                println!("Ambient Light: {}", data);
                Ok(value as u16)
            }
        }
    }

    fn writeRegister(&mut self, reg_addr: u16, lsb: u16, msb: u16) {
        let mut buffer = [0u8; 2];
        buffer[0] = reg_addr as u8; // 1バイト目にレジスタアドレスをセット
        buffer[1] = lsb as u8; // 2バイト目にLSBをセット
        buffer[2] = msb as u8; // 3バイト目にMSBをセット

        /* I2C-writeメッセージを送信 */
        self.device.write(&buffer)?;

        println!("Wrote to register 0x{:02x}:", reg_addr);
        Ok(()) // 構文を確認
    }
}

impl Sensor for VCNL4004Sensor {
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>> {
        let value = self.readRegister(VCNL4040Address::AlsData);
        let value = value * 0.1; // TODO: luxの計算があっているか確認
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