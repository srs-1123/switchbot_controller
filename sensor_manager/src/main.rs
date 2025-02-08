mod sensor_manager;
mod sensors;

use sensor_manager::{SensorManager, SensorId};
use sensors::vcnl4040::VCNL4040Sensor;
use sensors::hcsr5015::HCSR5015Sensor;

use std::os::unix::net::UnixStream;
use std::io::prelude::*;

/// 送信メッセージを管理するEnum
enum Message {
    On,     // 0x01
    Off,   // 0x02
}

impl Message {
    /// メッセージをバイナリデータに変換
    fn to_bytes(&self) -> u8 {
        match self {
            Message::On => 0x01,
            Message::Off => 0x02,
        }
    }
}

fn send_message(socket_path: &str , message: Message ) {
    // ソケットと接続
    if let Ok(mut stream) = UnixStream::connect(socket_path) {

        // データを送信
        let data = message.to_bytes();
        if let Err(e) = stream.write_all(&[data]) {
            eprintln!("Failed to send data: {}", e);
        } else {
            println!("Message sent successfully: {:?}", data);
        }
    } else {
        eprintln!("Failed to connect to the socket.");
    }
}

fn main() {
    let socket_path = "/tmp/light_control.sock"; // light_managerと共通化

    let mut sensor_manager = SensorManager::new();

    let mut vcnl_sensor = VCNL4040Sensor::new("/dev/i2c-1", 0x60).unwrap();
    if let Err(e) = vcnl_sensor.set_als_config(0x00) {
        eprintln!("設定失敗: {:?}", e);
    }

    sensor_manager.start_sensor(SensorId::VCNL4040, Box::new(vcnl_sensor));
    sensor_manager.start_sensor(SensorId::HCSR5015, Box::new(HCSR5015Sensor::new().unwrap()));

    // TODO: loopを終了させる処理追加
    // TODO: 送信はスレッドにさせてもいいかも
    loop {
        for sensor_thread in &sensor_manager.sensor_threads {
            if let Ok(data) = sensor_thread.receiver.recv() {
                match sensor_thread.sensor_id {
                    SensorId::VCNL4040 => {
                        println!("Received from VCNL4040: {}", data);
                        if data >= 50 {
                            send_message(socket_path, Message::Off);
                        } else {
                            send_message(socket_path, Message::On);
                        }
                    }
                    SensorId::HCSR5015 => {
                        println!("Received from sensor HCSR5015: {}", data);
                    }
                }
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    sensor_manager.stop_sensor(SensorId::VCNL4040);
    sensor_manager.stop_sensor(SensorId::HCSR5015);
}
