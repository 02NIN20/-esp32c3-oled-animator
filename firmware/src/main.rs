mod anim_frames;

use anyhow::{anyhow, Result};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    i2c::{config::Config, I2cDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::sys::link_patches;
use ssd1306::{
    mode::BufferedGraphicsMode,
    prelude::*,
    rotation::DisplayRotation,
    size::DisplaySize72x40,
    I2CDisplayInterface,
    Ssd1306,
};
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};

fn main() -> Result<()> {
    link_patches();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        pins.gpio5,
        pins.gpio6,
        &Config::new().baudrate(400_000u32.into()),
    )?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display: Ssd1306<_, DisplaySize72x40, BufferedGraphicsMode<_>> =
        Ssd1306::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

    display.init().map_err(|e| anyhow!("init OLED: {:?}", e))?;

    loop {
        for frame in anim_frames::FRAMES.iter() {
            display.clear(BinaryColor::Off).unwrap();
            let raw = ImageRaw::<BinaryColor>::new(frame, 72);
            Image::new(&raw, Point::zero())
                .draw(&mut display)
                .map_err(|e| anyhow!("draw: {:?}", e))?;
            display.flush().map_err(|e| anyhow!("flush: {:?}", e))?;
            FreeRtos::delay_ms(50);   // 20 fps
        }
    }
}
