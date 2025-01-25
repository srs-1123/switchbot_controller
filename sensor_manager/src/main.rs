extern crate i2cdev;

use std::error::Error;
use gpiod;
use i2cdev::core::*;
use i2cdev::linux;
// use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};

use std::thread;

#[derive(Clone)]
struct SensorData {
    sensor_id: usize,
    value: u16,
}

trait Sensor {
    fn collect_data(&mut self) -> Result<SensorData, Box<dyn Error>>; // TODO: なぜヒープに格納するのか
}


#[repr(u8)] 
enum VCNL4040Address {
    AlsConf = 0x00,
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

    fn setAlsConfig(&mut self, cmd: u8) -> Result<(), linux::LinuxI2CError>{
        self.writeRegister(VCNL4040Address::AlsConf as u8, cmd, 0x00)
    }

    fn readRegister(&mut self, reg_addr: u8) -> Result<u16, linux::LinuxI2CError> { 
        let mut buffer = [0u8; 2];

        // レジスタアドレスを送信
        self.device.write(&reg_addr.to_be_bytes())?; // TODO: 構文を確認

        // データを受信
        self.device.read(&mut buffer)?;

        // 16ビット値として返す
        let data = u16::from_be_bytes(buffer); // TODO: 関数の意味
        Ok(data)

        }

    fn writeRegister(&mut self, reg_addr: u8, lsb: u8, msb: u8) -> Result<(), linux::LinuxI2CError>{
        let mut buffer = [0u8; 3];
        buffer[0] = reg_addr as u8; // 1バイト目にレジスタアドレスをセット
        buffer[1] = lsb as u8; // 2バイト目にLSBをセット
        buffer[2] = msb as u8; // 3バイト目にMSBをセット

        /* I2C-writeメッセージを送信 */
        self.device.write(&buffer)?;

        println!("Wrote to register 0x{:02x}:", reg_addr);
        Ok(()) // 構文を確認
    }
}

impl Sensor for VCNL4040Sensor {
    fn collect_data(&mut self) -> Result<SensorData, Box<dyn Error + 'static>> {
        let value = self.readRegister(VCNL4040Address::AlsData as u8)?; 
        let value = value as f32 * 0.1; // TODO: luxの計算があっているか確認,
                                 // TODO: Result型とfloat型を計算している
        println!("Ambient Light: {}", value);
        let sensor_data = SensorData {
            sensor_id: 1,
            value: value as u16,
        };
        Ok(sensor_data)
    }
}

struct HCSR5015Sensor {
    chip: gpiod::Chip,
    line: gpiod::Lines<gpiod::Input>,
}

impl HCSR5015Sensor {
    fn new() -> Result<HCSR5015Sensor, Box<dyn Error + 'static>> {
        let chip = gpiod::Chip::new("/dev/gpiochip0")?;
        let options = gpiod::Options::input([17])
            .edge(gpiod::EdgeDetect::Both)
            .consumer("my-gpio-app"); // どのアプリが GPIO を使っているかを記録
        let line = chip.request_lines(options).expect("Failed to request lines");

        Ok ( Self { chip, line })

    }
}

impl Sensor for HCSR5015Sensor {
    fn collect_data(&mut self) -> Result<SensorData, Box<dyn Error>> {
        let mut values = 0u16;
        self.line.get_values(&mut values)?;
        let sensor_data = SensorData {
            sensor_id: 2,
            value: values as u16,
        };
        Ok(sensor_data)
    }
}

// センサー管理構造体
struct SensorManager {
    sensor_threads: Vec<SensorThreadStatus>,
    sender: Sender<SensorData>,  // センサーデータ送信用のチャネル
}

struct SensorThreadStatus {
    sensor_id: usize,
    is_running: bool,
}

impl SensorManager {
    fn new(sender: Sender<SensorData>) -> Self {
        Self {
            sensor_threads: Vec::new(),
            sender,
        }
    }

    fn start_sensor(&mut self, sensor_id: usize, sensor: Box<dyn Sensor + Send>) {
        // センサースレッドを起動し、SensorThreadStatusを更新
        let (tx, rx): (Sender<SensorData>, Receiver<SensorData>) = channel();
        let sensor_clone = sensor.clone();

        // TODO
        // is_runningにアクセスできる形に修正
        // sensor_idとセンサーデータをまとめてsenderで送る形に修正
        let status = SensorThreadStatus {
            sensor_id,
            is_running: true,
        };
        self.sensor_threads.push(status);

        thread::spawn(move || {
            loop {
                let data = sensor_clone.collect_data();
                tx.send(data).expect("Failed to send sensor data");
                // ここでsleepなどを挟んで定期的にデ}ータを収集
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        // 受信処理
        thread::spawn(move || {
            for data in rx {
                // 受信したデータをSensorManagerに送信
                self.sender.send(data).expect("Failed to send data to manager");
            }
        });
    }

    fn stop_sensor(&mut self, sensor_id: usize) {
        // 対象のセンサースレッドを停止し、フラグを更新
        if let Some(sensor) = self.sensor_threads.iter_mut().find(|s| s.sensor_id == sensor_id) {
            sensor.is_running = false;
        }
    }
}

fn main() {
    let (tx, rx): (Sender<SensorData>, Receiver<SensorData>) = channel();

    let mut sensor_manager = SensorManager::new(tx);

    // TODO: sensorのインスタンス生成
    // センサー1を開始
    sensor_manager.start_sensor(1, Box::new(VCNL4040Sensor));

    // メインスレッドでセンサーデータを受信して処理
    thread::spawn(move || {
        for data in rx {
            println!("Received from sensor {}: {}", data.sensor_id, data.value);
        }
    });

    // 少し待ってからセンサーを停止
    thread::sleep(std::time::Duration::from_secs(5));
    sensor_manager.stop_sensor(1);
}