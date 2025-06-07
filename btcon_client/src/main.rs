#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;
// Used for allocations
extern crate alloc;
extern crate flipperzero_alloc;
use alloc::string::ToString;
use flipperzero::{error, furi::string::FuriString, info};

use core::{ffi::CStr, time::Duration};

use bt::BluetoothApp;
use flipperzero::{furi::thread::sleep, notification::NotificationApp};
use flipperzero_rt::{entry, manifest};

use crate::{app::BtconnApp, bt::serial_profile::SerialProfileParams};

pub mod app;
mod bt;
mod screens;
mod utils;

// Define the FAP Manifest for this application
manifest!(
    name = "BTConn client",
    stack_size = 1024,
    app_version = 1,
    has_icon = false,
    icon = "icon10px.icon",
);

// Define the entry function
entry!(main);

fn real_main() -> anyhow::Result<()> {
    let notifications = NotificationApp::open();
    let bt = BluetoothApp::open();
    let mut app = BtconnApp { bt, notifications };
    let mut profile = app.bt.serial_profile_start(SerialProfileParams {
        adv_name: "Little btcon",
        appearance_char: Some(0x0040),
        service_uuid: Some(0x110A),
        mac_xor: 0x123,
    });

    profile.drop_event_callback();
    app.bt.start_advertising();
    app.bt.set_status_change_callback(
        |status, _ctx| {
            match status {
                flipperzero_sys::BtStatusOff => {
                    info!("BT is off NOW");
                    // Handle Bluetooth connected event
                }
                flipperzero_sys::BtStatusAdvertising => {
                    info!("BT is advertising");
                    // Handle Bluetooth disconnected event
                }
                flipperzero_sys::BtStatusConnected => {
                    info!("BT is connected");
                    // Handle Bluetooth connected event
                }
                flipperzero_sys::BtStatusUnavailable => {
                    info!("BT is unavailable");
                    // Handle Bluetooth unavailable event
                }
                _ => {}
            }
        },
        &mut 0,
    )?;
    profile.set_event_callback(
        128,
        |status, _ctx| {
            match status.event {
                flipperzero_sys::SerialServiceEventTypeDataSent => {
                    info!("Sent {} bytes of data", status.data.size);
                    // Handle sent data here
                }
                flipperzero_sys::SerialServiceEventTypeDataReceived => {
                    info!("Received {} bytes of data", status.data.size);
                    // Handle received data here
                }
                _ => {
                    unreachable!();
                }
            }
            return 0;
        },
        &mut 0,
    )?;

    let thread_handle = flipperzero::furi::thread::spawn(|| 0);
    thread_handle.join();

    sleep(Duration::from_secs(1));

    Ok(())
}

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    match real_main() {
        Ok(_) => 0,
        Err(e) => {
            let err_descr = FuriString::from(e.to_string().as_str());
            error!("Error: {}", err_descr);
            1
        }
    }
}
