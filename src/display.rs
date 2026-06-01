use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::gpio::{Gpio4, Gpio5};
use esp_idf_hal::units::Hertz;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, mode::BufferedGraphicsMode};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

pub struct Display<'a> {
    oled: Ssd1306<I2CInterface<I2cDriver<'a>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
}

impl<'a> Display<'a> {
    pub fn new(i2c0: I2C0<'a>, sda: Gpio4<'a>, scl: Gpio5<'a>) -> anyhow::Result<Self> {
        let config = I2cConfig::new().baudrate(Hertz(400_000));
        let i2c = I2cDriver::new(i2c0, sda, scl, &config)?;
        let interface = I2CDisplayInterface::new(i2c);
        let mut oled = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        oled.init().unwrap();
        
        Ok(Self { oled })
    }

    pub fn show_message(&mut self, line1: &str, line2: &str, line3: &str) {
        self.oled.clear(BinaryColor::Off).unwrap();
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(line1, Point::new(0, 10), text_style, Baseline::Top).draw(&mut self.oled).unwrap();
        Text::with_baseline(line2, Point::new(0, 30), text_style, Baseline::Top).draw(&mut self.oled).unwrap();
        Text::with_baseline(line3, Point::new(0, 50), text_style, Baseline::Top).draw(&mut self.oled).unwrap();
        
        self.oled.flush().unwrap();
    }
}