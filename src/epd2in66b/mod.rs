use core::fmt::Debug;
use core::marker::PhantomData;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::{InputPin, OutputPin};

mod command;
use command::Command;

pub struct Display<SPI, CS, BUSY, DC, RST, DELAY> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// DELAY
    _delay: PhantomData<DELAY>,
    /// CS for SPI
    cs: CS,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Resetting
    rst: RST,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Display<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin<Error: Debug>,
    DELAY: DelayUs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // hardware reset
        self.rst.set_low().unwrap();
        delay.delay_us(200);
        self.rst.set_high().unwrap();
        delay.delay_us(200);
        self.wait_for_busy_low();

        // software reset
        self.cmd(spi, Command::SWReset)?;
        self.wait_for_busy_low();

        Ok(())
    }

    pub fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let mut epd = Self {
            _spi: PhantomData,
            _delay: PhantomData,
            cs,
            busy,
            dc,
            rst,
        };

        epd.init(spi, delay)?;
        Ok(epd)
    }

    fn wait_for_busy_low(&self) {
        while let Ok(true) = self.busy.is_high() {}
    }

    fn cmd(&mut self, spi: &mut SPI, cmd: Command) -> Result<(), SPI::Error> {
        let _ = self.cs.set_low();
        let _ = self.dc.set_low();
        spi.write(&[cmd.address()])?;
        let _ = self.cs.set_high();
        Ok(())
    }
    fn data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        let _ = self.cs.set_low();
        let _ = self.dc.set_high();
        spi.write(data)?;
        let _ = self.cs.set_high();
        Ok(())
    }

    pub fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let _ = self.cs.set_low();

        self.cmd(spi, Command::WriteRAMBlackWhite)?;
        // self.data(spi, include_bytes!("picture.out"))?;
        self.data(spi, &[0xFF; 5624])?;

        self.cmd(spi, Command::WriteRAMRed)?;
        self.data(spi, &[0x00; 5624])?;
        // self.data(spi, include_bytes!("pictureR.out"))?;

        self.cmd(spi, Command::MasterActivation)?;
        self.wait_for_busy_low();

        let _ = self.cs.set_high();
        Ok(())
    }

    pub fn deep_sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.cmd(spi, Command::DeepSleepMode)
    }
}
