#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]

use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::{bail, Result};
use log::*;

use embedded_svc::utils::anyerror::*;
use embedded_svc::wifi::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;

use esp_idf_hal::{
    prelude::*,
    delay::*,
    ulp,
};

use test;

use esp_idf_svc::wifi::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::ping;

use esp_idf_sys;
use esp_idf_sys::esp;
use dht_sensor::*;

use gmqtt::{
    control_packet::{
        connect::{ConnectProperties, Login},
        publish::{PublishProperties},
        Connect, Packet, Publish, Subscribe,
    },
    read_packet, write_packet,
    Pid, QoS, QosPid, SubscribeFilter
};

include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));
#[allow(dead_code)]
const ULP: &[u8] = include_bytes!(env!("EMBUILD_GENERATED_BIN_FILE"));

const MQTT_PACKET_SIZE: u32 = 512;
const MQTT_ENDPOINT: &str = "192.168.178.100:1883";
const SSID: &str = "Honeypot";
const PASS: &str = "delicateBEAVER$$";

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // let peripherals = Peripherals::take().unwrap();
    // let pins = peripherals.pins;

    // // initialize the DHT11
    // let mut dht_pin = pins.gpio1.into_input_output_od()?;
    // let mut delay = esp_idf_hal::delay::Ets;
    // dht_pin.set_high()?;
    // delay.delay_ms(1000_u16);

    // initialize wifi/net stack
    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let mut wifi = wifi(
        netif_stack,
        sys_loop_stack,
        default_nvs,
    )?;

    // set up TCP stream

    thread::Builder::new().name("mqtt_test".to_string()).stack_size(16 * 1024).spawn(move || {
        let mut stream = TcpStream::connect(MQTT_ENDPOINT).unwrap();
        let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
        let data = "hello world!".as_bytes();
    
        mqtt_connect(&mut stream, &mut buffer);
        mqtt_connack(&mut stream, &mut buffer);
        mqtt_sub(&mut stream, &mut buffer, "esp32s2/test/temp");
        // mqtt_pub(&mut stream, &mut buffer, "esp32s2/test/temp", data);
        mqtt_puback(&mut stream, &mut buffer);
    });

    loop {
        // info!("Test??");
        // let (temp, rh) = match dht11::Reading::read(&mut delay, &mut dht_pin) {
        //     Ok(dht11::Reading {
        //         temperature,
        //         relative_humidity,
        //     }) => {
        //         (temperature, relative_humidity)
        //     },
        //     Err(e) => (0, 0),
        //     // Err(e) => bail!("Something went wrong: {:?}", e),
        // };

        // info!("T: {}, H: {}", temp, rh);

        // thread::sleep(Duration::from_millis(1500));
        // sleep_deep(&Duration::from_millis(120));
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[inline(never)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    // info!("Wifi configuration set, about to get status");

    // let status = wifi.get_status();

    // if let Status(
    //     ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
    //     ApStatus::Started(ApIpStatus::Done),
    // ) = status
    // {
    //     info!("Wifi connected");

    //     ping(&ip_settings)?;
    // } else {
    //     bail!("Unexpected Wifi status: {:?}", status);
    // }

    Ok(wifi)
}

#[inline(never)]
fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

    Ok(())
}

#[inline(never)]
fn mqtt_connect(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<()> {
    // let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
    // send CONNECT
    // buffer.iter_mut().for_each(|x| *x = 0);
    info!("About to send a Connect packet...");
    let mqtt_connect_login = Login {
        username: "admin",
        password: "tmehdd23UDH7p".as_bytes(),
    };
    let mqtt_connect_properties = ConnectProperties {
        topic_alias_max: Some(0),
        max_packet_size: Some(MQTT_PACKET_SIZE),
        ..ConnectProperties::default()
    };
    let mqtt_connect_packet = Packet::Connect(Connect {
        protocol: gmqtt::Protocol::V5,
        clean_start: false,
        keep_alive: 60_u16,
        client_id: "esp32s2-test1111",
        last_will: None,
        login: Some(mqtt_connect_login),
        properties: Some(mqtt_connect_properties),
    });
    let len = write_packet(&mqtt_connect_packet, buffer).unwrap();
    info!("{:?}", mqtt_connect_packet);
    stream.write_all(buffer);

    stream.flush();
    
    Ok(())
}

#[inline(never)]
fn mqtt_connack(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<()> {
    // let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
    // receive CONNACK
    // buffer.iter_mut().for_each(|x| *x = 0);
    stream.read(buffer);
    let mut ack = read_packet(buffer).unwrap();
    info!("Connected successfully!");
    info!("{:?}", ack);
    stream.flush();
    
    Ok(())
}

#[inline(never)]
fn mqtt_pub(stream: &mut TcpStream, buffer: &mut [u8], topic: &str, data: &[u8]) -> Result<()> {
    // let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
    // buffer.iter_mut().for_each(|x| *x = 0);
    info!("About to send a Publish packet...");
    let mqtt_pub_properties = PublishProperties {
        payload_format_indicator: Some(0x01),
        ..PublishProperties::default()
    };
    let mqtt_pub_packet = Packet::Publish(Publish {
        dup: false,
        qospid: QosPid::AtMostOnce,
        retain: false,
        topic: topic,
        properties: Some(mqtt_pub_properties),
        payload: data,
    });
    let len = write_packet(&mqtt_pub_packet, buffer).unwrap();
    info!("{:?}", mqtt_pub_packet);
    stream.write_all(buffer);
    info!("After stream.write_all()");

    stream.flush();

    Ok(())
}

#[inline(never)]
fn mqtt_puback(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<()> {
    // let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
    // buffer.iter_mut().for_each(|x| *x = 0);
    stream.read(buffer);
    let mut ack = read_packet(buffer).unwrap();
    info!("Published successfully!");
    info!("{:?}", ack);
    stream.flush();
    Ok(())
}

#[inline(never)]
fn mqtt_sub(stream: &mut TcpStream, buffer: &mut [u8], topic: &str) -> Result<()> {
    // let mut buffer = [0u8; MQTT_PACKET_SIZE as usize];
    let mut filters: gmqtt::Vec<SubscribeFilter, 4> = gmqtt::Vec::new();
    filters.push(
        SubscribeFilter::new(topic, QoS::AtMostOnce)
    );
    let mqtt_pub_packet = Packet::Subscribe(Subscribe {
        pid: Pid::new(1234_u16),
        filters: filters,
        properties: None,
    });
    let len = write_packet(&mqtt_pub_packet, buffer).unwrap();
    stream.write_all(buffer);
    stream.flush();
    Ok(())
}

#[allow(dead_code)]
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

// fn blinky<T>() -> Result<()>  {
//     let mut led_pin = pins.gpio11.into_output()?;
//     load_ulp(ULP)?;
//     let mut led_count: u32;
//     loop {
//         info!("LED Count (Main): {}", read_rtc_slow::<u32>(LED_COUNT_MAIN));
//         info!("LED Count (ULP):  {}", read_rtc_slow::<u32>(LED_COUNT_ULP));
//         for _ in 1..=5 {
//             led_pin.set_high()?;
//             thread::sleep(Duration::from_millis(250));
//             led_pin.set_low()?;
//             thread::sleep(Duration::from_millis(250));

//             led_count = read_rtc_slow(LED_COUNT_MAIN);
//             write_rtc_slow(LED_COUNT_MAIN, led_count + 1);
//         }

//         info!("About to sleep for 10 seconds...");
//         sleep_deep(&Duration::from_secs(10))?;
//     }
// }

#[allow(dead_code)]
fn sleep_deep(sleep_for: &Duration) -> Result<()> {
    use esp_idf_hal::ulp;

    unsafe {
        esp_idf_sys::esp_deep_sleep(sleep_for.as_micros() as u64);
    }

    Ok(())
}

#[allow(dead_code)]
fn read_rtc_slow<T>(var: *mut core::ffi::c_void) -> T {
    unsafe {
        core::ptr::read_volatile(var as *mut T)
    }
}

#[allow(dead_code)]
fn write_rtc_slow<T>(var: *mut core::ffi::c_void, val: T) {
    unsafe {
        core::ptr::write_volatile(var as *mut T, val);
    }
}
