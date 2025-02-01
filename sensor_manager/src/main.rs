mod sensor_manager;
mod sensors;

use sensor_manager::SensorManager;
use sensors::vcnl4040::VCNL4040Sensor;
use sensors::hcsr5015::HCSR5015Sensor;

fn main() {
    let mut sensor_manager = SensorManager::new();

    let mut vcnl_sensor = VCNL4040Sensor::new("/dev/i2c-1", 0x60).unwrap();
    if let Err(e) = vcnl_sensor.set_als_config(0x00) {
        eprintln!("設定失敗: {:?}", e);
    }

    sensor_manager.start_sensor(1, Box::new(vcnl_sensor));
    sensor_manager.start_sensor(2, Box::new(HCSR5015Sensor::new().unwrap()));

    loop {
        for sensor_thread in &sensor_manager.sensor_threads {
            if let Ok(data) = sensor_thread.receiver.recv() {
                println!("Received from sensor {}: {}", sensor_thread.sensor_id, data);
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    sensor_manager.stop_sensor(1);
    sensor_manager.stop_sensor(2);
}
