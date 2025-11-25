use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi;
use embedded_hal::spi::SpiDevice;

/**
A simple SPI device that owns the entire SPI bus. You should probably use a
generic SPI device if you can, but `rp2040_hal` does not provide one so I made
this as a substitute.
*/
pub struct ExclusiveSpiDevice<SPI, CS, Timer> {
    spi: SPI,
    cs: CS,
    timer: Timer,
}

impl<SPI, CS, Timer> ExclusiveSpiDevice<SPI, CS, Timer> {
    pub fn new(spi: SPI, cs: CS, timer: Timer) -> Self {
        Self { spi, cs, timer }
    }
}

impl<SPI, CS, Timer, E> spi::ErrorType for ExclusiveSpiDevice<SPI, CS, Timer>
where
    SPI: spi::SpiBus<Error = E>,
    CS: OutputPin<Error = E>,
    E: spi::Error,
{
    type Error = E;
}

impl<SPI, CS, Timer, E> SpiDevice for ExclusiveSpiDevice<SPI, CS, Timer>
where
    SPI: spi::SpiBus<Error = E>,
    CS: OutputPin<Error = E>,
    Timer: DelayNs,
    E: spi::Error,
{
    fn transaction(
        &mut self,
        operations: &mut [spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            self.cs.set_low()?;
            match operation {
                spi::Operation::Read(words) => self.spi.read(words)?,
                spi::Operation::Write(words) => self.spi.write(words)?,
                spi::Operation::Transfer(read, write) => self.spi.transfer(read, write)?,
                spi::Operation::TransferInPlace(words) => self.spi.transfer_in_place(words)?,
                spi::Operation::DelayNs(ns) => self.timer.delay_ns(*ns),
            }
            self.cs.set_high()?;
        }

        Ok(())
    }
}
