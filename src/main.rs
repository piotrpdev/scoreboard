use esp_idf_svc::{
    hal::{
        delay::FreeRtos,
        gpio::{self, Gpio0, PinDriver},
        prelude::*,
        spi::{SpiConfig, SpiDeviceDriver, SpiDriver, config::DriverConfig},
    },
    log::EspLogger,
};

use anyhow::{Context, Result, anyhow};

use embedded_graphics::{
    image::{Image, ImageRaw, ImageRawLE},
    pixelcolor::{Rgb565, raw::LittleEndian},
    prelude::*,
};

use log::info;
use st7735_lcd::{Orientation, ST7735};

const SPI_BAUDRATE_MHZ: u32 = 80;
const DISPLAY_WIDTH: u32 = 128;
const DISPLAY_HEIGHT: u32 = 160;
const DISPLAY_USE_RGB: bool = true;
const DISPLAY_INVERT_COLORS: bool = false;

static RAW_FERRIS_IMAGE: ImageRawLE<'static, Rgb565> =
    ImageRaw::new(include_bytes!("../assets/ferris.raw"), 86);
static FERRIS_IMAGE: Image<'static, ImageRaw<'static, Rgb565, LittleEndian>> =
    Image::new(&RAW_FERRIS_IMAGE, Point::new(26, 8));

type Spi<'a> = SpiDeviceDriver<'a, SpiDriver<'a>>;
type DcPin<'a> = PinDriver<'a, gpio::Gpio2, gpio::Output>;
type RstPin<'a> = PinDriver<'a, gpio::Gpio3, gpio::Output>;

type Display<'a> = ST7735<Spi<'a>, DcPin<'a>, RstPin<'a>>;

fn create_display_instance() -> Result<Display<'static>> {
    let peripherals =
        Peripherals::take().context("Failed to take peripherals when creating display instance")?;

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo = peripherals.pins.gpio4;
    let cs = peripherals.pins.gpio1;

    let spi = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<Gpio0>::None,
        Some(cs),
        &DriverConfig::default(),
        &SpiConfig::new().baudrate(SPI_BAUDRATE_MHZ.MHz().into()),
    )?;

    let rst =
        PinDriver::output(peripherals.pins.gpio3).context("Failed to create RST pin driver")?;
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

    let mut display = create_display_instance().context("Failed to create display instance")?;

    display
        .init(&mut FreeRtos)
        .map_err(|()| anyhow!("Failed to initialize display"))?;
    display
        .clear(Rgb565::BLACK)
        .map_err(|()| anyhow!("Failed to clear display"))?;
    display
        .set_orientation(&Orientation::LandscapeSwapped)
        .map_err(|()| anyhow!("Failed to set display orientation"))?;
    display.set_offset(0, 25);

    FERRIS_IMAGE
        .draw(&mut display)
        .map_err(|()| anyhow!("Failed to draw image on display"))?;

    info!("LCD test done.");
    loop {
        FreeRtos::delay_ms(1000);
    }
}
