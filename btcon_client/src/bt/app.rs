use core::{
    ffi::{c_void, CStr},
    time::Duration,
};

use alloc::boxed::Box;
use flipperzero::{furi::thread::sleep, info};
use flipperzero_sys::{
    bt_disconnect, bt_forget_bonded_devices, bt_profile_restore_default,
    bt_set_status_changed_callback, furi::UnsafeRecord, furi_hal_bt_start_advertising,
    furi_hal_bt_stop_advertising, Bt,
};

use crate::bt::{callbacks::bt_status_changed_callback, serial_profile::SerialProfile};

use super::serial_profile::{self, SerialProfileParams};

pub struct BluetoothApp {
    record: UnsafeRecord<Bt>,
    event_callback: Option<*mut c_void>,
}

impl BluetoothApp {
    pub const NAME: &CStr = c"bt";

    pub fn open() -> Self {
        BluetoothApp {
            record: unsafe { UnsafeRecord::open(Self::NAME) },
            event_callback: None,
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

    pub fn serial_profile_start(&self, params: SerialProfileParams) -> SerialProfile {
        let profile_base = unsafe { serial_profile::setup_profile(&self.record, params) };
        SerialProfile::new(profile_base)
    }

    pub fn set_status_change_callback<'a, F, Ctx>(
        &mut self,
        mut callback: F,
        ctx: &'a mut Ctx,
    ) -> anyhow::Result<()>
    where
        F: FnMut(flipperzero_sys::BtStatus, &'a mut Ctx) + 'a,
    {
        // Get the previous callback to drop it later.
        let callback_wrapper = crate::utils::CallbackWrapper::new(&mut callback, ctx);
        let wrapper_ptr = callback_wrapper as *mut _ as *mut c_void;
        self.event_callback = Some(wrapper_ptr);
        unsafe {
            if self.as_ptr().is_null() {
                info!("Profile base is null, cannot set event callback");
                anyhow::bail!("Profile base is null");
            }
            bt_set_status_changed_callback(
                self.as_ptr(),
                Some(bt_status_changed_callback::<'a, F, Ctx>),
                wrapper_ptr,
            )
        }
        Ok(())
    }

    pub fn drop_event_callback(&mut self) {
        if let Some(callback_ptr) = self.event_callback.take() {
            drop(unsafe { Box::from_raw(callback_ptr) })
        }
        unsafe {
            bt_set_status_changed_callback(self.as_ptr(), None, core::ptr::null_mut());
        }
    }

    pub fn restore_profile(&self) -> bool {
        let status = unsafe { bt_profile_restore_default(self.as_ptr()) };
        // Wait 2nd core to update nvm storage.
        sleep(Duration::from_millis(200));
        status
    }
}

impl Drop for BluetoothApp {
    fn drop(&mut self) {
        self.drop_event_callback();
        self.disconnect();
        self.restore_profile();
    }
}
