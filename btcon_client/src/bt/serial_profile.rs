use core::ffi::{c_char, c_void, CStr};

use flipperzero::{furi::sync::Mutex, info};
use flipperzero_sys::{
    ble_svc_battery_start, ble_svc_battery_stop, ble_svc_dev_info_start, ble_svc_dev_info_stop,
    ble_svc_serial_set_callbacks, ble_svc_serial_start, ble_svc_serial_stop, bt_profile_start,
    furi::UnsafeRecord, furi_hal_version_get_ble_local_device_name_ptr,
    furi_hal_version_get_ble_mac, BleServiceBattery, BleServiceDevInfo, BleServiceSerial, Bt,
    FuriHalBleProfileBase, FuriHalBleProfileParams, FuriHalBleProfileTemplate, GapConfig,
    GapConfig__bindgen_ty_1, GapConnectionParamsRequest, GapPairingPinCodeVerifyYesNo,
    SerialServiceEvent,
};

use alloc::boxed::Box;

use crate::{bt::callbacks, utils::CallbackWrapper};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SerialProfileParams {
    pub adv_name: &'static str,
    pub appearance_char: Option<u16>,
    pub service_uuid: Option<u16>,
    pub mac_xor: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BlePorfileSerial {
    pub base: FuriHalBleProfileBase,
    pub dev_info_svc: *mut BleServiceDevInfo,
    pub battery_svc: *mut BleServiceBattery,
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
        Service_UUID_16: 0x110A,
        Service_UUID_128: [0; 16],
    },
    appearance_char: 0x0040,
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
        dev_info_svc: ble_svc_dev_info_start(),
        battery_svc: ble_svc_battery_start(true),
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
    ble_svc_dev_info_stop(profile.dev_info_svc);
    ble_svc_battery_stop(profile.battery_svc);
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
    config.mac_address[0] ^= params_ref.mac_xor as u8;
    config.mac_address[1] ^= (params_ref.mac_xor >> 8) as u8;
    info!("Setting mac address to: {:?}", config.mac_address);

    // Get neccesary name flags
    let device_name =
        CStr::from_ptr(furi_hal_version_get_ble_local_device_name_ptr()).to_bytes()[0];
    config.adv_name = [0; 18];
    config.adv_name[0] = device_name as c_char; // Set the first character to the device name
    for (id, char) in params_ref.adv_name.chars().enumerate() {
        let index = id + 1;
        if index < config.adv_name.len() {
            config.adv_name[index] = char as c_char;
        } else {
            break; // Prevent overflow
        }
    }
    config.adv_name[config.adv_name.len() - 1] = 0x00; // Null-terminate the string
    if let Some(appearance) = &params_ref.appearance_char {
        config.appearance_char = *appearance;
    }
    config.adv_service.UUID_Type = 0x01;
    if let Some(service_uuid) = &params_ref.service_uuid {
        config.adv_service.Service_UUID_16 = *service_uuid;
    }
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
    event_callback: Mutex<Option<*mut c_void>>,
}

impl SerialProfile {
    pub fn new(profile_base: *mut FuriHalBleProfileBase) -> Self {
        SerialProfile {
            profile_base,
            event_callback: Mutex::new(None),
        }
    }

    pub fn as_ptr(&self) -> *mut FuriHalBleProfileBase {
        self.profile_base
    }

    pub fn set_event_callback<F>(
        &mut self,
        buffer_size: u16,
        mut callback: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(SerialServiceEvent) -> u16,
    {
        // Get the previous callback to drop it later.
        self.drop_event_callback();

        let callback_wrapper = CallbackWrapper::new(&mut callback);
        let wrapper_ptr = callback_wrapper as *mut _ as *mut c_void;
        let mut callback_guard = self.event_callback.lock();
        *callback_guard = Some(wrapper_ptr);

        unsafe {
            if self.profile_base.is_null() {
                info!("Profile base is null, cannot set event callback");
                anyhow::bail!("Profile base is null");
            }
            let serial_svc = (*(self.profile_base as *mut BlePorfileSerial)).serial_svc;
            ble_svc_serial_set_callbacks(
                serial_svc,
                buffer_size,
                Some(callbacks::ble_svc_serial_callback::<F>),
                wrapper_ptr,
            );
        }
        Ok(())
    }

    pub fn drop_event_callback(&mut self) {
        let mut callback_guard = self.event_callback.lock();
        unsafe {
            let serial_svc = (*(self.profile_base as *mut BlePorfileSerial)).serial_svc;
            if !serial_svc.is_null() {
                ble_svc_serial_set_callbacks(serial_svc, 0, None, core::ptr::null_mut());
            }
        }
        if let Some(previous) = callback_guard.take() {
            drop(unsafe { Box::from_raw(previous) });
        }
    }
}

impl Drop for SerialProfile {
    fn drop(&mut self) {
        self.drop_event_callback();
    }
}
