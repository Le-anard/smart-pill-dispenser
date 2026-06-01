use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, TIMER1, CHANNEL1};
use esp_idf_hal::gpio::Gpio7;
use esp_idf_hal::units::Hertz;

pub struct Buzzer<'a> {
    driver: LedcDriver<'a>,
    max_duty: u32,
}

impl<'a> Buzzer<'a> {
    pub fn new(timer: TIMER1<'a>, channel: CHANNEL1<'a>, pin: Gpio7<'a>) -> anyhow::Result<Self> {
        let timer_driver = LedcTimerDriver::new(timer, &TimerConfig::new().frequency(Hertz(2000)))?;
        let driver = LedcDriver::new(channel, timer_driver, pin)?;
        let max_duty = driver.get_max_duty();
        Ok(Self { driver, max_duty })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.driver.set_duty(self.max_duty / 2)?; // 50% duty cycle for sound
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.driver.set_duty(0)?;
        Ok(())
    }
}