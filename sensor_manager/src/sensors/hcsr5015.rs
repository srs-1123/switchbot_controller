use std::error::Error;
use gpiod;
use crate::sensors::Sensor;

pub struct HCSR5015Sensor {
    line: gpiod::Lines<gpiod::Input>,
}

impl HCSR5015Sensor {
    pub fn new() -> Result<HCSR5015Sensor, Box<dyn Error>> {
        let chip = gpiod::Chip::new("/dev/gpiochip0")?;
        let options = gpiod::Options::input([17])
            .edge(gpiod::EdgeDetect::Both)
            .consumer("my-gpio-app");
        let line = chip.request_lines(options)?;
        Ok(Self { line })
    }
}

impl Sensor for HCSR5015Sensor {
    fn collect_data(&mut self) -> Result<u16, Box<dyn Error>> {
        let mut values = 0u16;
        self.line.get_values(&mut values)?;
        println!("value: {:?}", values);
        Ok(values)
    }
}
