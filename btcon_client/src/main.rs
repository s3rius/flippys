#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;
// Used for allocations
extern crate alloc;
extern crate flipperzero_alloc;

use core::{ffi::CStr, time::Duration};

use bt::BluetoothApp;
use flipperzero::{
    furi::thread::sleep,
    notification::{led, NotificationApp},
};
use flipperzero_rt::{entry, manifest};

mod bt;
mod screens;
mod utils;

// Define the FAP Manifest for this application
manifest!(
    name = "BTConn client",
    stack_size = 1024,
    app_version = 1,
    has_icon = true,
    icon = "icon10px.icon",
);

// Define the entry function
entry!(main);

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    // Print a message to the console
    // let mut dialog = DialogsApp::open();
    let mut notification_app = NotificationApp::open();

    let bt = BluetoothApp::open();

    bt.disconnect();
    bt.stop_advertising();
    bt.forget_bonded_devices();
    let profile = bt.serial_profile_start();
    profile.bt().start_advertising();

    notification_app.notify_blocking(&led::BLINK_START_BLUE);

    sleep(Duration::from_secs(10));

    notification_app.notify_blocking(&led::BLINK_STOP);

    profile.restore();

    sleep(Duration::from_secs(1));

    0
}
