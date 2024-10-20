//! A driver for the Waveshare three-color E-ink Pi Pico hat 'Pico-ePaper-2.66-B' B/W/R.
//!
//!
//! This driver was built and tested for this 296x152, 2.66inch three-color E-Ink display hat for the Pi Pico, it is expected to work for
//! other boards too, but that might depend on how the OTP memory in the display is programmed by the factory.
//!
//! The driver embedded in the display of this board is the SSD1675B, [documented by cursedhardware](https://cursedhardware.github.io/epd-driver-ic/SSD1675B.pdf).
//!
//! The pin assigments are shown on the Waveshare wiki [schematic](https://www.waveshare.com/w/upload/8/8d/Pico-ePaper-2.66.pdf).
//!
//! Information on this display/hat can be found at the [Waveshare Wiki](https://www.waveshare.com/wiki/Pico-ePaper-2.66-B).
//! Do read this documentation, in particular to understand how often this display both should and should not be updated.
//!
//! # Example for the 'Pico-ePaper-2.66-B' B/W/R Pi Pico Hat E-Ink Display
//! This example was created in an environment using the [Knurling](https://github.com/knurling-rs) ```flip-link```, ```defmt``` and ```probe-run``` tools - you will
//! need to adjust for your preferred setup.
//!```ignore
//!#![no_std]
//!#![no_main]
//!use epd_waveshare::{epd2in66b::*, prelude::*};
//!
//!use cortex_m_rt::entry;
//!//use defmt::*;
//!use defmt_rtt as _;
//!use panic_probe as _;
//!
//!// Use embedded-graphics to create a bitmap to show
//!fn drawing() -> Display2in66b {
//!    use embedded_graphics::{
//!        mono_font::{ascii::FONT_10X20, MonoTextStyle},
//!        prelude::*,
//!        primitives::PrimitiveStyle,
//!        text::{Alignment, Text},
//!    };
//!
//!    // Create a Display buffer to draw on, specific for this ePaper
//!    let mut display = Display2in66b::default();
//!
//!    // Landscape mode, USB plug to the right
//!    display.set_rotation(DisplayRotation::Rotate270);
//!
//!    // Change the background from the default black to white
//!    let _ = display
//!        .bounding_box()
//!        .into_styled(PrimitiveStyle::with_fill(TriColor::White))
//!        .draw(&mut display);
//!
//!    // Draw some text on the buffer
//!    let text = "Pico-ePaper-2.66 B/W/R";
//!    Text::with_alignment(
//!        text,
//!        display.bounding_box().center() + Point::new(1, 0),
//!        MonoTextStyle::new(&FONT_10X20, TriColor::Black),
//!        Alignment::Center,
//!    )
//!    .draw(&mut display)
//!    .unwrap();
//!    Text::with_alignment(
//!        text,
//!        display.bounding_box().center() + Point::new(0, 1),
//!        MonoTextStyle::new(&FONT_10X20, TriColor::Chromatic),
//!        Alignment::Center,
//!    )
//!    .draw(&mut display)
//!    .unwrap();
//!
//!    display
//!}
//!
//!#[entry]
//!fn main() -> ! {
//!    use fugit::RateExtU32;
//!    use rp_pico::hal::{
//!        self,
//!        clocks::{init_clocks_and_plls, Clock},
//!        gpio::{FunctionSpi, PinState, Pins},
//!        pac,
//!        sio::Sio,
//!        watchdog::Watchdog,
//!    };
//!
//!    // Boilerplate to access the peripherals
//!    let mut pac = pac::Peripherals::take().unwrap();
//!    let core = pac::CorePeripherals::take().unwrap();
//!    let mut watchdog = Watchdog::new(pac.WATCHDOG);
//!    let external_xtal_freq_hz = 12_000_000u32;
//!    let clocks = init_clocks_and_plls(
//!        external_xtal_freq_hz,
//!        pac.XOSC,
//!        pac.CLOCKS,
//!        pac.PLL_SYS,
//!        pac.PLL_USB,
//!        &mut pac.RESETS,
//!        &mut watchdog,
//!    )
//!    .ok()
//!    .unwrap();
//!    let sio = Sio::new(pac.SIO);
//!    let pins = Pins::new(
//!        pac.IO_BANK0,
//!        pac.PADS_BANK0,
//!        sio.gpio_bank0,
//!        &mut pac.RESETS,
//!    );
//!
//!    // Pin assignments of the Pi Pico-ePaper-2.66 Hat
//!    let _ = pins.gpio10.into_mode::<FunctionSpi>();
//!    let _ = pins.gpio11.into_mode::<FunctionSpi>();
//!    let chip_select_pin = pins.gpio9.into_push_pull_output_in_state(PinState::High);
//!    let is_busy_pin = pins.gpio13.into_floating_input();
//!    let data_or_command_pin = pins.gpio8.into_push_pull_output_in_state(PinState::High);
//!    let reset_pin = pins.gpio12.into_push_pull_output_in_state(PinState::High);
//!
//!    // SPI
//!    let spi = hal::Spi::<_, _, 8>::new(pac.SPI1);
//!    let mut spi = spi.init(
//!        &mut pac.RESETS,
//!        clocks.peripheral_clock.freq(),
//!        20_000_000u32.Hz(), // The SSD1675B docs say 20MHz max
//!        &SPI_MODE,
//!    );
//!
//!    // Delay
//!    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
//!
//!    // Setup the EPD driver
//!    let mut e_paper = Epd2in66b::new(
//!        &mut spi,
//!        chip_select_pin,
//!        is_busy_pin,
//!        data_or_command_pin,
//!        reset_pin,
//!        &mut delay,
//!        None,
//!    )
//!    .unwrap();
//!
//!    // Create and fill a Display buffer
//!    let display = drawing();
//!
//!    // Send the Display buffer to the ePaper RAM
//!    e_paper
//!        .update_color_frame(
//!            &mut spi,
//!            &mut delay,
//!            &display.bw_buffer(),
//!            &display.chromatic_buffer(),
//!        )
//!        .unwrap();
//!
//!    // Render the ePaper RAM - takes time.
//!    e_paper.display_frame(&mut spi, &mut delay).unwrap();
//!
//!    // Always turn off your EPD as much as possible - ePaper wears out while powered on.
//!    e_paper.sleep(&mut spi, &mut delay).unwrap();
//!
//!    delay.delay_ms(60 * 1000);
//!
//!    // Set the display all-white before storing your ePaper long-term.
//!    e_paper.wake_up(&mut spi, &mut delay).unwrap();
//!    e_paper.clear_frame(&mut spi, &mut delay).unwrap();
//!    e_paper.display_frame(&mut spi, &mut delay).unwrap();
//!    e_paper.sleep(&mut spi, &mut delay).unwrap();
//!
//!    loop {}
//!}
//!```
use core::fmt::Debug;

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::color::TriColor;
use crate::interface::DisplayInterface;
use crate::prelude::ErrorKind;
use crate::traits::{
    ErrorType, InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

pub(crate) mod command;
use self::command::*;
use crate::buffer_len;

/// Display height in pixels.
pub const WIDTH: u32 = 152;
/// Display width in pixels
pub const HEIGHT: u32 = 296;

const SINGLE_BYTE_WRITE: bool = true;

/// White, display this during long-term storage
pub const DEFAULT_BACKGROUND_COLOR: TriColor = TriColor::White;

/// A Display buffer configured with our extent and color depth.
#[cfg(feature = "graphics")]
pub type Display2in66b = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) * 2 },
    TriColor,
>;

/// The EPD 2in66-B driver.
pub struct Epd2in66b<SPI, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    background: TriColor,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd2in66b<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug,
    DC: OutputPin,
    DC::Error: Copy + Debug,
    RST: OutputPin,
    RST::Error: Copy + Debug,
{
    type Error = ErrorKind<SPI, BUSY, DC, RST>;
}

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd2in66b<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug,
    DC: OutputPin,
    DC::Error: Copy + Debug,
    RST: OutputPin,
    RST::Error: Copy + Debug,
{
    async fn init(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        // We follow the sequence of the Pi-Pico hat example code.
        self.hw_reset(spi).await?;
        self.sw_reset(spi).await?;
        self.data_entry_mode(spi, DataEntryRow::XMinor, DataEntrySign::IncYIncX)
            .await?;
        self.set_display_window(spi, 0, 0, WIDTH - 1, HEIGHT - 1)
            .await?;
        self.update_control1(
            spi,
            WriteMode::Normal,
            WriteMode::Normal,
            OutputSource::S8ToS167,
        )
        .await?;
        self.set_cursor(spi, 0, 0).await?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST>
    for Epd2in66b<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug,
    DC: OutputPin,
    DC::Error: Copy + Debug,
    RST: OutputPin,
    RST::Error: Copy + Debug,
{
    async fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_achromatic_frame(spi, black).await?;
        self.update_chromatic_frame(spi, chromatic).await
    }

    async fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
    ) -> Result<(), Self::Error> {
        self.set_cursor(spi, 0, 0).await?;
        self.interface.cmd(spi, Command::WriteBlackWhiteRAM).await?;
        self.interface.data(spi, black).await
    }

    async fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.set_cursor(spi, 0, 0).await?;
        self.interface.cmd(spi, Command::WriteRedRAM).await?;
        self.interface.data(spi, chromatic).await
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd2in66b<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug,
    DC: OutputPin,
    DC::Error: Copy + Debug,
    RST: OutputPin,
    RST::Error: Copy + Debug,
{
    type DisplayColor = TriColor;

    async fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay_us: Option<u32>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut epd = Self {
            interface: DisplayInterface::new(busy, dc, rst, delay_us),
            background: DEFAULT_BACKGROUND_COLOR,
        };
        epd.init(spi).await?;
        Ok(epd)
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface
            .cmd_with_data(
                spi,
                Command::DeepSleepMode,
                &[DeepSleep::SleepLosingRAM as u8],
            )
            .await
    }

    async fn wake_up(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.init(spi).await
    }

    fn set_background_color(&mut self, color: Self::DisplayColor) {
        self.background = color;
    }

    fn background_color(&self) -> &Self::DisplayColor {
        &self.background
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    async fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.set_cursor(spi, 0, 0).await?;
        self.update_achromatic_frame(spi, buffer).await?;
        self.red_pattern(spi, PatW::W160, PatH::H296, StartWith::Zero)
            .await // do NOT consider background here since red overrides other colors
    }

    async fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        self.set_display_window(spi, x, y, x + width, y + height)
            .await?;
        self.set_cursor(spi, x, y).await?;
        self.update_achromatic_frame(spi, buffer).await?;
        self.set_display_window(spi, 0, 0, WIDTH, HEIGHT).await
    }

    async fn display_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface.cmd(spi, Command::MasterActivation).await?;
        self.wait_until_idle(spi).await
    }

    async fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_frame(spi, buffer).await?;
        self.display_frame(spi).await
    }

    async fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        let (white, red) = match self.background {
            TriColor::Black => (StartWith::Zero, StartWith::Zero),
            TriColor::White => (StartWith::One, StartWith::Zero),
            TriColor::Chromatic => (StartWith::Zero, StartWith::One),
        };
        self.black_white_pattern(spi, PatW::W160, PatH::H296, white)
            .await?;
        self.red_pattern(spi, PatW::W160, PatH::H296, red).await
    }

    async fn set_lut(
        &mut self,
        _spi: &mut SPI,
        __refresh_rate: Option<RefreshLut>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_until_idle(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await
    }
}

// Helper functions that enforce some type and value constraints. Meant to help with code readability. They caught some of my silly errors -> yay rust!.
impl<SPI, BUSY, DC, RST> Epd2in66b<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug,
    DC: OutputPin,
    DC::Error: Copy + Debug,
    RST: OutputPin,
    RST::Error: Copy + Debug,
{
    async fn wait_until_idle(
        &mut self,
        spi: &mut SPI,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.wait_until_idle(spi, false).await
    }

    async fn hw_reset(
        &mut self,
        spi: &mut SPI,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        // The initial delay is taken from other code here, the 2 ms comes from the SSD1675B datasheet.
        self.interface.reset(spi, 20_000, 2_000).await?;
        self.wait_until_idle(spi).await
    }
    async fn sw_reset(
        &mut self,
        spi: &mut SPI,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.cmd(spi, Command::Reset).await?;
        self.wait_until_idle(spi).await
    }
    async fn data_entry_mode(
        &mut self,
        spi: &mut SPI,
        row: DataEntryRow,
        sign: DataEntrySign,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(spi, Command::DataEntryMode, &[row as u8 | sign as u8])
            .await
    }
    async fn set_display_window(
        &mut self,
        spi: &mut SPI,
        xstart: u32,
        ystart: u32,
        xend: u32,
        yend: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(
                spi,
                Command::SetXAddressRange,
                &[(((xstart >> 3) & 0x1f) as u8), (((xend >> 3) & 0x1f) as u8)],
            )
            .await?;
        self.interface
            .cmd_with_data(
                spi,
                Command::SetYAddressRange,
                &[
                    ((ystart & 0xff) as u8),
                    (((ystart >> 8) & 0x01) as u8),
                    ((yend & 0xff) as u8),
                    (((yend >> 8) & 0x01) as u8),
                ],
            )
            .await
    }

    async fn update_control1(
        &mut self,
        spi: &mut SPI,
        red_mode: WriteMode,
        bw_mode: WriteMode,
        source: OutputSource,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(
                spi,
                Command::DisplayUpdateControl1,
                &[((red_mode as u8) << 4 | bw_mode as u8), (source as u8)],
            )
            .await
    }

    async fn set_cursor(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(spi, Command::SetXAddressCounter, &[((x >> 3) & 0x1f) as u8])
            .await?;
        self.interface
            .cmd_with_data(
                spi,
                Command::SetYAddressCounter,
                &[((y & 0xff) as u8), (((y >> 8) & 0x01) as u8)],
            )
            .await
    }

    async fn black_white_pattern(
        &mut self,
        spi: &mut SPI,
        w: PatW,
        h: PatH,
        phase: StartWith,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(
                spi,
                Command::BlackWhiteRAMTestPattern,
                &[phase as u8 | h as u8 | w as u8],
            )
            .await?;
        self.wait_until_idle(spi).await
    }

    async fn red_pattern(
        &mut self,
        spi: &mut SPI,
        w: PatW,
        h: PatH,
        phase: StartWith,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd_with_data(
                spi,
                Command::RedRAMTestPattern,
                &[phase as u8 | h as u8 | w as u8],
            )
            .await?;
        self.wait_until_idle(spi).await
    }
}
