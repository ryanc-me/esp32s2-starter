#![allow(unused_imports)]

use std::{thread, time::*};

use anyhow::{bail, Result};
use log::*;

use embedded_svc::utils::anyerror::*;

use esp_idf_hal::prelude::*;
use esp_idf_hal::ulp;

use esp_idf_sys;
use esp_idf_sys::esp;


include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));
const ULP: &[u8] = include_bytes!(env!("EMBUILD_GENERATED_BIN_FILE"));


fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let mut led_pin = pins.gpio11.into_output()?;

    load_ulp(ULP)?;
    let mut led_count: u32;
    loop {
        for _ in 1..=5 {
            led_pin.set_high()?;
            thread::sleep(Duration::from_millis(250));
            led_pin.set_low()?;
            thread::sleep(Duration::from_millis(250));

            led_count = read_rtc_slow(LED_COUNT_MAIN);
            write_rtc_slow(LED_COUNT_MAIN, led_count + 1);
        }

        info!("LED Count (Main): {}", read_rtc_slow::<u32>(LED_COUNT_MAIN));
        info!("LED Count (ULP):  {}", read_rtc_slow::<u32>(LED_COUNT_ULP));
        info!("About to sleep for 10 seconds...");
        sleep_deep(&Duration::from_secs(10))?;
    }

    #[allow(unreachable_code)]
    Ok(())
}

fn load_ulp(ulp_data: &[u8]) -> Result<()> {
    use esp_idf_hal::ulp;

    // only load/start the ULP once, when the MCU first boots
    if !read_rtc_slow::<bool>(ULP_LOADED) {
        unsafe {
            esp!(esp_idf_sys::ulp_riscv_load_binary(
                ulp_data.as_ptr(),
                ulp_data.len() as _
            ))?;
            info!("Loaded the ULP binary ({} bytes)", ulp_data.len());
            ulp::enable_timer(false);
            esp!(esp_idf_sys::ulp_riscv_run())?;
            esp!(esp_idf_sys::esp_sleep_enable_ulp_wakeup())?;
            write_rtc_slow(ULP_LOADED, true);
            info!("Started the ULP coprocessor");
        }
    }

    Ok(())
}

fn sleep_deep(sleep_for: &Duration) -> Result<()> {
    use esp_idf_hal::ulp;

    unsafe {
        esp_idf_sys::esp_deep_sleep(sleep_for.as_micros() as u64);
    }

    Ok(())
}

fn read_rtc_slow<T>(var: *mut core::ffi::c_void) -> T {
    unsafe {
        core::ptr::read_volatile(var as *mut T)
    }
}

fn write_rtc_slow<T>(var: *mut core::ffi::c_void, val: T) {
    unsafe {
        core::ptr::write_volatile(var as *mut T, val);
    }
}
