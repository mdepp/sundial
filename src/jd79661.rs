use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;

type CommandData<'a> = (u8, &'a [u8]);

#[allow(clippy::upper_case_acronyms)]
enum Command<'a> {
    Misc(u8, &'a [u8]),
    PSR(&'a [u8; 2]),
    PWR(&'a [u8; 6]),
    POF,
    PON,
    BTST(&'a [u8; 7]),
    DSLP,
    DTM(&'a [u8]),
    DSP, // XXX there is supposed to be an output parameter to read
    DRF(&'a [u8; 1]),
    AUTO(&'a [u8; 1]),
    PLL(&'a [u8; 1]),
    CDI(&'a [u8; 1]),
    TRES(&'a [u8; 4]),
}

impl<'a> From<&'a Command<'a>> for CommandData<'a> {
    fn from(value: &'a Command) -> Self {
        use Command::*;
        match value {
            Misc(c, d) => (*c, d),
            PSR(d) => (0x00, d.as_slice()),
            PWR(d) => (0x01, d.as_slice()),
            POF => (0x02, &[0x00]),
            PON => (0x04, &[]),
            BTST(d) => (0x06, d.as_slice()),
            DSLP => (0x07, &[0xA5]),
            DTM(d) => (0x10, d),
            DSP => (0x11, &[]),
            DRF(d) => (0x12, d.as_slice()),
            AUTO(d) => (0x17, d.as_slice()),
            PLL(d) => (0x30, d.as_slice()),
            CDI(d) => (0x50, d.as_slice()),
            TRES(d) => (0x61, d.as_slice()),
        }
    }
}

const START_SEQUENCE: &[Command] = &[
    Command::Misc(0x4D, &[0x78]),
    Command::PSR(&[0x8F, 0x29]), // PSR, Display resolution is 128x250; scan up first line G1->G2, shift right first data S1->S2
    Command::PWR(&[0x07, 0x00, 0, 0, 0, 0]), // PWR
    Command::Misc(0x03, &[0x10, 0x54, 0x44]), // POFS
    Command::BTST(&[0x05, 0x00, 0x3F, 0x0A, 0x25, 0x12, 0x1A]),
    Command::CDI(&[0x37]),              // CDI
    Command::Misc(0x60, &[0x02, 0x02]), // TCON
    Command::TRES(&[0, 128, 0, 250]),   // TRES
    Command::Misc(0xE7, &[0x1C]),
    Command::Misc(0xE3, &[0x22]),
    Command::Misc(0xB4, &[0xD0]),
    Command::Misc(0xB5, &[0x03]),
    Command::Misc(0xE9, &[0x01]),
    Command::PLL(&[0x08]),
    Command::PON,
];

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 250;
pub const PIXDEPTH: usize = 2;

pub struct JD79661<SPI, DC, RST, BUSY>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    spi: SPI,
    dc_pin: DC,
    rst_pin: RST,
    busy_pin: BUSY,
}

impl<SPI, DC, RST, BUSY, E> JD79661<SPI, DC, RST, BUSY>
where
    SPI: SpiDevice<Error = E>,
    DC: OutputPin<Error = E>,
    RST: OutputPin<Error = E>,
    BUSY: InputPin<Error = E>,
    E: embedded_hal::digital::Error,
{
    pub fn new(spi: SPI, dc_pin: DC, rst_pin: RST, busy_pin: BUSY) -> Result<Self, E> {
        Ok(Self {
            spi,
            dc_pin,
            rst_pin,
            busy_pin,
        })
    }

    pub fn hardware_reset(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        self.rst_pin.set_high()?;
        timer.delay_ms(20);
        self.rst_pin.set_low()?;
        timer.delay_ms(40);
        self.rst_pin.set_high()?;
        timer.delay_ms(50);

        Ok(())
    }

    fn busy_wait(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        while self.busy_pin.is_low()? {
            timer.delay_ms(10);
        }

        Ok(())
    }

    pub fn power_up(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        self.hardware_reset(timer)?;
        self.busy_wait(timer)?;

        timer.delay_ms(10);
        self.command_list(START_SEQUENCE)?;
        self.busy_wait(timer)?;

        Ok(())
    }

    pub fn power_down(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        self.command_list(&[Command::POF])?;
        self.busy_wait(timer)?;
        self.command_list(&[Command::DSLP])?;
        timer.delay_ms(100);

        Ok(())
    }

    pub fn update(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        self.command_list(&[Command::DRF(&[0x00])])?;
        self.busy_wait(timer)?;

        Ok(())
    }

    pub fn update_sleep(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        // PON -> DRF -> POF
        self.command_list(&[Command::AUTO(&[0xA5])])?;
        self.busy_wait(timer)?;

        Ok(())
    }

    pub fn update_deepsleep(&mut self, timer: &mut impl DelayNs) -> Result<(), E> {
        // PON -> DRF -> POF -> DSLP
        self.command_list(&[Command::AUTO(&[0xA7])])?;
        self.busy_wait(timer)?;

        Ok(())
    }

    fn command_list(&mut self, commands: &[Command]) -> Result<(), E> {
        for command in commands {
            let (c, d) = CommandData::from(command);

            self.dc_pin.set_low()?;
            self.spi.write(&[c])?;

            self.dc_pin.set_high()?;
            self.spi.write(d)?;
        }

        Ok(())
    }

    pub fn write_buffer(&mut self, buffer: &[u8; WIDTH * HEIGHT * PIXDEPTH / 8]) -> Result<(), E> {
        self.command_list(&[Command::DTM(buffer.as_slice()), Command::DSP])
    }
}
