use core::{ffi::CStr, time::Duration};

use flipperzero::{furi::thread::sleep, info};
use flipperzero_sys::{
    bt_disconnect, bt_forget_bonded_devices, bt_profile_restore_default, furi::UnsafeRecord,
    furi_hal_bt_start_advertising, furi_hal_bt_stop_advertising, Bt,
};

use crate::bt::serial_profile::SerialProfile;

use super::serial_profile::{self, SerialProfileParams};

pub struct BluetoothApp {
    record: UnsafeRecord<Bt>,
}

impl BluetoothApp {
    pub const NAME: &CStr = c"bt";

    pub fn open() -> Self {
        BluetoothApp {
            record: unsafe { UnsafeRecord::open(Self::NAME) },
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut Bt {
        self.record.as_ptr()
    }

    pub fn disconnect(&self) {
        unsafe { bt_disconnect(self.as_ptr()) };
        // Wait 2nd core to update nvm storage.
        sleep(Duration::from_millis(200));
    }

    pub fn stop_advertising(&self) {
        unsafe { furi_hal_bt_stop_advertising() }
    }

    pub fn start_advertising(&self) {
        unsafe { furi_hal_bt_start_advertising() }
    }

    pub fn forget_bonded_devices(&self) {
        unsafe { bt_forget_bonded_devices(self.as_ptr()) }
    }

    pub fn serial_profile_start(self) -> SerialProfile {
        let profile_base = unsafe {
            serial_profile::setup_profile(&self.record, SerialProfileParams { max_xor: 0x15 })
        };
        SerialProfile::new(self, profile_base)
    }

    pub fn restore_profile(&self) -> bool {
        let pointer = &serial_profile::PROFILE_BASE_CONFIG.stop as *const Option<_>;
        info!("{}", pointer as usize);
        let status = unsafe { bt_profile_restore_default(self.as_ptr()) };
        // Wait 2nd core to update nvm storage.
        sleep(Duration::from_millis(200));
        status
    }
}
