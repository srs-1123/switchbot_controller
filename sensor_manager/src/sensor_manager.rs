use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use crate::sensors::Sensor;

#[derive(PartialEq)] // 意味を調べる
pub enum SensorId {
    VCNL4040,
    HCSR5015,
}

pub struct SensorThreadStatus {
    pub sensor_id: SensorId, // TODO: enum型に変更
    pub receiver: Receiver<u16>, 
    is_running: bool,
    thread_handle: Option<thread::JoinHandle<()>>,
}

pub struct SensorManager {
    pub sensor_threads: Vec<SensorThreadStatus>,
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            sensor_threads: Vec::new(),
        }
    }

    pub fn start_sensor(&mut self, sensor_id: SensorId, mut sensor: Box<dyn Sensor>) {
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
            thread_handle: Some(handle),
        };

        self.sensor_threads.push(status);
    }

    pub fn stop_sensor(&mut self, sensor_id: SensorId) {
        if let Some(sensor_thread) = self.sensor_threads.iter_mut().find(|s| s.sensor_id == sensor_id) {
            sensor_thread.is_running = false;
            if let Some(handle) = sensor_thread.thread_handle.take() {
                handle.join().expect("Failed to join thread");
            }
            self.sensor_threads.retain(|s| s.sensor_id != sensor_id);
        }
    }
}
