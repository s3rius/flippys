use core::ffi::{c_void, CStr};

use flipperzero::info;
use flipperzero_sys::{
    ble_svc_serial_start, ble_svc_serial_stop, bt_profile_start, furi::UnsafeRecord,
    furi_hal_version_get_ble_local_device_name_ptr, furi_hal_version_get_ble_mac,
    furi_hal_version_get_hw_color, BleServiceSerial, Bt, FuriHalBleProfileBase,
    FuriHalBleProfileParams, FuriHalBleProfileTemplate, GapConfig, GapConfig__bindgen_ty_1,
    GapConnectionParamsRequest, GapPairingPinCodeVerifyYesNo,
};

use alloc::boxed::Box;

use crate::bt::BluetoothApp;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SerialProfileParams {
    pub max_xor: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BlePorfileSerial {
    pub base: FuriHalBleProfileBase,
    pub serial_svc: *mut BleServiceSerial,
}

pub const PROFILE_BASE_CONFIG: FuriHalBleProfileTemplate = FuriHalBleProfileTemplate {
    start: Some(start_profile),
    stop: Some(stop_profile),
    get_gap_config: Some(get_gap_config),
};

const GAP_CONFIG: GapConfig = GapConfig {
    adv_name: [0; 18],
    adv_service: GapConfig__bindgen_ty_1 {
        UUID_Type: 0x01,
        Service_UUID_16: 0x1800,
        Service_UUID_128: [0; 16],
    },
    appearance_char: 0x0180,
    mfg_data: [0; 23],
    mfg_data_len: 0,
    bonding_mode: true,
    pairing_method: GapPairingPinCodeVerifyYesNo,
    mac_address: [0; 6],
    conn_param: GapConnectionParamsRequest {
        conn_int_min: 0x06,
        conn_int_max: 0x24,
        slave_latency: 0,
        supervisor_timeout: 0,
    },
};

#[no_mangle]
unsafe extern "C" fn start_profile(_: FuriHalBleProfileParams) -> *mut FuriHalBleProfileBase {
    let profile = Box::new(BlePorfileSerial {
        base: FuriHalBleProfileBase {
            config: &PROFILE_BASE_CONFIG,
        },
        serial_svc: ble_svc_serial_start(),
    });

    let profile_ptr = Box::leak(profile);
    &mut profile_ptr.base
}

#[no_mangle]
unsafe extern "C" fn stop_profile(prof: *mut FuriHalBleProfileBase) {
    if prof.is_null() {
        info!("Profile pointer is null, skipping stop");
        return;
    }
    if (&*prof).config != &PROFILE_BASE_CONFIG {
        info!("Profile is not the expected one, skipping stop");
        return;
    }
    // Since we have leaked a box, we need to ensure we don't double free it.
    let profile = &mut *(prof as *mut BlePorfileSerial);
    ble_svc_serial_stop(profile.serial_svc);
}

#[no_mangle]
unsafe extern "C" fn get_gap_config(
    target_config: *mut GapConfig,
    profile_params: FuriHalBleProfileParams,
) {
    target_config.copy_from(&GAP_CONFIG, 1);
    let params_ref = &*(profile_params as *mut SerialProfileParams);
    let config = target_config
        .as_mut()
        .expect("Cannot dereference target_config");

    let ble_mac: *const u8 = furi_hal_version_get_ble_mac();
    config
        .mac_address
        .copy_from_slice(core::slice::from_raw_parts(
            ble_mac,
            core::mem::size_of::<[u8; 6]>(),
        ));
    config.mac_address[2] += 1;
    config.mac_address[0] ^= params_ref.max_xor as u8;
    config.mac_address[1] ^= (params_ref.max_xor >> 8) as u8;
    info!("Setting mac address to: {:?}", config.mac_address);

    // Get neccesary name flags
    let device_name =
        CStr::from_ptr(furi_hal_version_get_ble_local_device_name_ptr()).to_bytes()[0];
    config.adv_name = [0; 18];
    config.adv_name[0] = device_name;
    info!("Device name: {:?}", device_name);
    for (id, char) in b"Flip my shit".iter().enumerate() {
        let index = id + 1;
        if index < config.adv_name.len() {
            config.adv_name[index] = *char;
        } else {
            break; // Prevent overflow
        }
    }
    let color = furi_hal_version_get_hw_color();
    config.adv_service.UUID_Type = 0x01;
    config.adv_service.Service_UUID_16 |= color.0 as u16;
}

pub unsafe fn setup_profile(
    bt: &UnsafeRecord<Bt>,
    mut params: SerialProfileParams,
) -> *mut FuriHalBleProfileBase {
    bt_profile_start(
        bt.as_ptr(),
        &PROFILE_BASE_CONFIG,
        &mut params as *mut _ as *mut c_void,
    )
}

pub struct SerialProfile {
    profile_base: *mut FuriHalBleProfileBase,
    bt: BluetoothApp,
}

impl SerialProfile {
    pub fn new(bt: BluetoothApp, profile_base: *mut FuriHalBleProfileBase) -> Self {
        SerialProfile { profile_base, bt }
    }

    pub fn as_ptr(&self) -> *mut FuriHalBleProfileBase {
        self.profile_base
    }

    pub fn bt(&self) -> &BluetoothApp {
        &self.bt
    }

    pub fn restore(self) -> BluetoothApp {
        self.bt.disconnect();
        self.bt.restore_profile();
        self.bt
    }
}
