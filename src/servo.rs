use esp_idf_hal::ledc::{config::TimerConfig, config::Resolution, LedcDriver, LedcTimerDriver, TIMER0, CHANNEL0};
use esp_idf_hal::gpio::Gpio6;
use esp_idf_hal::units::Hertz;
use std::thread::sleep;
use std::time::Duration;

pub struct PillServo<'a> {
    driver: LedcDriver<'a>,
}

impl<'a> PillServo<'a> {
    pub fn new(timer: TIMER0<'a>, channel: CHANNEL0<'a>, pin: Gpio6<'a>) -> anyhow::Result<Self> {
        // Explicitly set resolution to Bits14 so the ESP32-C3 internal clock divider
        // can comfortably hit 50Hz without running out of register space.
        let config = TimerConfig::new()
            .frequency(Hertz(50))
            .resolution(Resolution::Bits14);

        let timer_driver = LedcTimerDriver::new(timer, &config)?;
        let driver = LedcDriver::new(channel, timer_driver, pin)?;
        Ok(Self { driver })
    }

    pub fn dispense(&mut self) -> anyhow::Result<()> {
        // get_max_duty() now dynamically returns 16384 instead of 256
        let max_duty = self.driver.get_max_duty();
        
        // Rotate to dispense (approx 180 degrees - 2ms pulse)
        // 10% of 16384 = 1638 ticks. (1638 / 16384 * 20ms total period = 2.0ms)
        let duty_dispense = (max_duty as f32 * 0.10) as u32; 
        self.driver.set_duty(duty_dispense)?;
        sleep(Duration::from_secs(2));

        // Return to original position (0 degrees - 1ms pulse)
        // 5% of 16384 = 819 ticks. (819 / 16384 * 20ms total period = 1.0ms)
        let duty_home = (max_duty as f32 * 0.05) as u32;
        self.driver.set_duty(duty_home)?;
        
        Ok(())
    }
}