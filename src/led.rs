use esp_idf_hal::gpio::{Gpio8, Gpio9, PinDriver, Output};

// Changed from LedPins to Leds to match your main.rs and the impl block below
pub struct Leds<'a> {
    pub red: PinDriver<'a, Output>,
    pub green: PinDriver<'a, Output>,
}

impl<'a> Leds<'a> {
  pub fn new(red_pin: Gpio8<'a>, green_pin: Gpio9<'a>) -> anyhow::Result<Self> {
        let red = PinDriver::output(red_pin)?;
        let green = PinDriver::output(green_pin)?;
        Ok(Self { red, green })
    }

    pub fn turn_off_all(&mut self) -> anyhow::Result<()> {
        self.red.set_low()?;
        self.green.set_low()?;
        Ok(())
    }
}