use std::sync::LazyLock;

use esp_idf_svc::{
    hal::{
        delay::FreeRtos,
        gpio::{self, Gpio0, PinDriver},
        prelude::*,
        spi::{self, SpiDeviceDriver, SpiDriver},
    },
    log::EspLogger,
};

use anyhow::{anyhow, Result};

use embedded_graphics::{
    image::{Image, ImageRaw, ImageRawLE},
    pixelcolor::{raw::LittleEndian, Rgb565},
    prelude::*,
};

use log::info;
use st7735_lcd::{Orientation, ST7735};

const SPI_BAUDRATE_MHZ: u32 = 80;
const DISPLAY_WIDTH: u32 = 128;
const DISPLAY_HEIGHT: u32 = 160;
const DISPLAY_USE_RGB: bool = true;
const DISPLAY_INVERT_COLORS: bool = false;

static RAW_FERRIS_IMAGE: LazyLock<ImageRawLE<'static, Rgb565>> =
    LazyLock::new(|| ImageRaw::new(include_bytes!("../assets/ferris.raw"), 86));

static FERRIS_IMAGE: LazyLock<Image<'static, ImageRaw<'static, Rgb565, LittleEndian>>> =
    LazyLock::new(|| Image::new(&RAW_FERRIS_IMAGE, Point::new(26, 8)));

type Spi<'a> = SpiDeviceDriver<'a, SpiDriver<'a>>;
type DcPin<'a> = PinDriver<'a, gpio::Gpio2, gpio::Output>;
type RstPin<'a> = PinDriver<'a, gpio::Gpio3, gpio::Output>;

type Display<'a> = ST7735<Spi<'a>, DcPin<'a>, RstPin<'a>>;

fn create_display_instance() -> Result<Display<'static>> {
    let peripherals = Peripherals::take()?;

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo = peripherals.pins.gpio4;
    let sdi = Option::<Gpio0>::None;
    let cs = Some(peripherals.pins.gpio1);

    let spi_config = spi::SpiConfig::new().baudrate(SPI_BAUDRATE_MHZ.MHz().into());

    let spi = spi::SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        sdi,
        cs,
        &Default::default(),
        &spi_config,
    )?;

    let rst = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio2)?;

    let display = ST7735::new(
        spi,
        dc,
        rst,
        DISPLAY_USE_RGB,
        DISPLAY_INVERT_COLORS,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
    );

    Ok(display)
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();

    let mut display = create_display_instance()?;

    display
        .init(&mut FreeRtos)
        .map_err(|_| anyhow!("Failed to initialize display"))?;
    display
        .clear(Rgb565::BLACK)
        .map_err(|_| anyhow!("Failed to clear display"))?;
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .map_err(|_| anyhow!("Failed to set orientation"))?;
    display.set_offset(0, 25);

    FERRIS_IMAGE
        .draw(&mut display)
        .map_err(|_| anyhow!("Failed to draw image"))?;

    info!("LCD test done.");
    loop {
        FreeRtos::delay_ms(1000);
    }
}
