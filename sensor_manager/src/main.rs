extern crate i2cdev;

use std::error::Error;
use gpiod;
use i2cdev::core::*;
use i2cdev::linux;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

trait Sensor: Send {  // Sendトレイトを追加
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>>; 
}

#[repr(u8)] 
enum VCNL4040Address {
    AlsData = 0x09,
}

struct VCNL4040Sensor {
    device: linux::LinuxI2CDevice,
}

impl VCNL4040Sensor {
    fn new(device_path: &str, address: u16) -> Result<VCNL4040Sensor, linux::LinuxI2CError> {
        let device = linux::LinuxI2CDevice::new(device_path, address)?;
        Ok(Self { device })
    }

    // fn set_als_config(&mut self, cmd: u8) -> Result<(), linux::LinuxI2CError> {
    //     self.write_register(0x00, cmd, 0x00)
    // }

    fn read_register(&mut self, reg_addr: u8) -> Result<u16, linux::LinuxI2CError> { 
        let mut buffer = [0u8; 2];
        self.device.write(&reg_addr.to_be_bytes())?;
        self.device.read(&mut buffer)?;
        let data = u16::from_be_bytes(buffer); 
        Ok(data)
    }
    // fn write_register(&mut self, reg_addr: u8, lsb: u8, msb: u8) -> Result<(), linux::LinuxI2CError> {
    //     let mut buffer = [0u8; 3];
    //     buffer[0] = reg_addr as u8;
    //     buffer[1] = lsb as u8;
    //     buffer[2] = msb as u8;
    //     self.device.write(&buffer)?;
    //     println!("Wrote to register 0x{:02x}:", reg_addr);
    //     Ok(())
    // }
}

impl Sensor for VCNL4040Sensor {
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>> {
        let value = self.read_register(VCNL4040Address::AlsData as u8)?; 
        let value = value as f32 * 0.1; 
        println!("Ambient Light: {}", value);
        Ok(value as u16)
    }
}

struct HCSR5015Sensor {
    // chip: gpiod::Chip,
    line: gpiod::Lines<gpiod::Input>,
}

impl HCSR5015Sensor {
    fn new() -> Result<HCSR5015Sensor, Box<dyn Error>> {
        let chip = gpiod::Chip::new("/dev/gpiochip0")?;
        let options = gpiod::Options::input([17])
            .edge(gpiod::EdgeDetect::Both)
            .consumer("my-gpio-app");
        let line = chip.request_lines(options).expect("Failed to request lines");

        // Ok(Self { chip, line })
        Ok(Self { line })
    }
}

impl Sensor for HCSR5015Sensor {
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>> {
        let mut values = 0u16;
        self.line.get_values(&mut values)?;
        Ok(values)
    }
}

struct SensorThreadStatus {
    sensor_id: usize,
    is_running: bool,
    receiver: Receiver<u16>, 
    thread_handle: Option<thread::JoinHandle<()>>,  // スレッドハンドルを追加
}

struct SensorManager {
    sensor_threads: Vec<SensorThreadStatus>,
}

impl SensorManager {
    fn new() -> Self {
        Self {
            sensor_threads: Vec::new(),
        }
    }

    fn start_sensor(&mut self, sensor_id: usize, mut sensor: Box<dyn Sensor>) {
        let (tx, rx): (Sender<u16>, Receiver<u16>) = channel();

        let handle = thread::spawn(move || {
            loop {
                let data = sensor.collect_data();
                if let Ok(data) = data {
                    tx.send(data).expect("Failed to send sensor data");
                }
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        let status = SensorThreadStatus {
            sensor_id,
            is_running: true,
            receiver: rx,
            thread_handle: Some(handle), // スレッドハンドルを保存
        };

        self.sensor_threads.push(status);
    }

    fn stop_sensor(&mut self, sensor_id: usize) {
        if let Some(sensor_thread) = self.sensor_threads.iter_mut().find(|s| s.sensor_id == sensor_id) {
            sensor_thread.is_running = false;
            // スレッドの終了を待つ
            if let Some(handle) = sensor_thread.thread_handle.take() {
                handle.join().expect("Failed to join thread");
            }
            self.sensor_threads.retain(|s| s.sensor_id != sensor_id); // Receiverを閉じる
        }
    }
}

fn main() {
    let mut sensor_manager = SensorManager::new();

    // センサー1を開始
    sensor_manager.start_sensor(1, Box::new(VCNL4040Sensor::new("/dev/i2c-1", 0x13).unwrap()));
    sensor_manager.start_sensor(2, Box::new(HCSR5015Sensor::new().unwrap()));

    // メインスレッドでセンサーデータを受信して処理
    for sensor_thread in &sensor_manager.sensor_threads {
        if let Ok(data) = sensor_thread.receiver.recv() {
            println!("Received from sensor {}: {}", sensor_thread.sensor_id, data);
        }
    }

    // 少し待ってからセンサーを停止
    thread::sleep(std::time::Duration::from_secs(5));
    sensor_manager.stop_sensor(1);
    sensor_manager.stop_sensor(2);
}
