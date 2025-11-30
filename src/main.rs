#![no_std]
#![no_main]

mod calendar;
mod exclusive_spi_device;
mod jd79661;
mod jd79661_display;
mod rtclock;
mod theme;

use calendar::moon;
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_8X13_BOLD;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::{Alignment, Text};
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use fugit::TimerInstantU64;

#[cfg(target_arch = "riscv32")]
use panic_halt as _;
#[cfg(target_arch = "arm")]
use panic_probe as _;

// Alias for our HAL crate
use hal::entry;

#[cfg(rp2350)]
use rp235x_hal as hal;

#[cfg(rp2040)]
use rp2040_hal as hal;

use hal::Spi;
use hal::fugit::RateExtU32;
use hal::gpio::FunctionSpi;

use crate::jd79661_display::JD79661Theme;
use crate::rtclock::RealTimeClock;
use crate::{
    exclusive_spi_device::ExclusiveSpiDevice, jd79661::JD79661, jd79661_display::JD79661Display,
};

// use bsp::entry;
// use bsp::hal;
// use rp_pico as bsp;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[unsafe(link_section = ".boot2")]
#[used]
#[cfg(rp2040)]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
#[cfg(rp2350)]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp2040 and rp235x peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[entry]
fn main() -> ! {
    let _ = _main();
    loop {}
}

struct TimerClock {
    reference_instant: TimerInstantU64<1_000_000>,
    reference_timestamp: u64,
    timer: hal::Timer,
}

impl TimerClock {
    fn new(timer: hal::Timer, timestamp: u64) -> Self {
        Self {
            reference_instant: timer.get_counter(),
            reference_timestamp: timestamp,
            timer,
        }
    }

    fn get_timestamp(&self) -> u64 {
        let d1 = self.timer.get_counter().duration_since_epoch().to_secs();
        let d2 = self.reference_instant.duration_since_epoch().to_secs();
        d1 - d2 + self.reference_timestamp
    }
}

impl rtclock::RealTimeClock for TimerClock {
    fn get_time(&self) -> rtclock::InstantSecs {
        rtclock::Instant::from_ticks(self.get_timestamp())
    }
}

// Do the actual logic in a helper function as it's more convenient to return
// a result
fn _main() -> Result<(), core::convert::Infallible> {
    info!("Program start");
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    #[cfg(rp2040)]
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    #[cfg(rp2350)]
    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let clock = TimerClock::new(timer, env!("BUILD_TIMESTAMP").parse().unwrap());

    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();
    let spi: Spi<_, _, _, 8> =
        Spi::new(pac.SPI0, (mosi, sclk)).init(&mut pac.RESETS, 12u32.MHz(), 4.MHz(), spi::MODE_0);

    let dc = pins.gpio6.into_push_pull_output();
    let rst = pins.gpio7.into_push_pull_output();
    let cs = pins.gpio8.into_push_pull_output();
    let busy = pins.gpio9.into_pull_down_input();

    let mut screen = JD79661::new(ExclusiveSpiDevice::new(spi, cs, timer), dc, rst, busy)?;

    screen.power_up(&mut timer)?;

    let mut display = JD79661Display::default();
    let theme = JD79661Theme::new();

    loop {
        draw_frame(&mut display, &theme, &clock)?;

        screen.write_buffer(display.buffer())?;
        screen.update_sleep(&mut timer)?;

        timer.delay_ms(1000 * 3600); // Wait an hour
    }
}

pub fn draw_frame<Color: PixelColor, Error>(
    draw_target: &mut impl DrawTarget<Color = Color, Error = Error>,
    theme: &impl theme::Theme<Color = Color>,
    clock: &impl RealTimeClock,
) -> Result<(), Error> {
    draw_target
        .bounding_box()
        .into_styled(PrimitiveStyle::with_fill(theme.background()))
        .draw(draw_target)?;

    let moon_phase = moon::get_phase(clock.get_time());
    let moon_phase_label = moon::get_phase_label(moon_phase);
    let moon_illumination = moon::get_illumination(moon_phase);

    let mut buf = [0u8; 64];
    let text = format_no_std::show(
        &mut buf,
        format_args!(
            "Phase {:02.0}%\nIllum {:02.0}%\n{}",
            moon_phase * 100.0,
            moon_illumination * 100.0,
            moon_phase_label
        ),
    )
    .unwrap();

    Text::with_alignment(
        text,
        draw_target.bounding_box().center(),
        MonoTextStyle::new(&FONT_8X13_BOLD, theme.text()),
        Alignment::Center,
    )
    .draw(draw_target)?;

    Ok(())
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"JD79661 Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

// End of file
