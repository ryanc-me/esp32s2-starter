#![no_std]
#![no_main]
#![feature(core_intrinsics, start)]

use esp_idf_hal::delay;
use esp_idf_hal::prelude::*;

extern crate panic_halt;

#[no_mangle]
static mut ULP_LOADED: bool = false;
#[no_mangle]
static mut LED_COUNT_ULP: u32 = 0;
#[no_mangle]
static mut LED_COUNT_MAIN: u32 = 0;

#[no_mangle]
fn main() {
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut delay = delay::Ulp;
    let mut led = pins.gpio2.into_output_od().unwrap();

    loop {
        led.set_high().unwrap();
        delay.delay_ms(100_u32);

        led.set_low().unwrap();
        delay.delay_ms(100_u32);

        unsafe {
            let led_count = core::ptr::read_volatile(&LED_COUNT_ULP);
            core::ptr::write_volatile(&mut LED_COUNT_ULP, led_count + 1);
        }
    }
}
