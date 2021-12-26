#![allow(unused_imports)]

use anyhow::bail;
use log::*;

use embedded_svc::utils::anyerror::*;

use esp_idf_hal::prelude::*;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let mut led_pin = pins.gpio11.into_output()?;
    led_pin.set_high()?;

    Ok(())
}