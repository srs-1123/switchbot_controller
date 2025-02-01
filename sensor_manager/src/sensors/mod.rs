use std::error::Error;
pub mod vcnl4040;
pub mod hcsr5015;

pub trait Sensor: Send {  // Sendトレイトを追加
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>>; 
}