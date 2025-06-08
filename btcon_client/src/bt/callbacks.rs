use core::ffi::c_void;

use flipperzero_sys::{BtStatus, SerialServiceEvent};

use crate::utils::CallbackWrapper;

#[allow(unused)]
pub unsafe extern "C" fn ble_svc_serial_callback<TF>(
    status: SerialServiceEvent,
    data: *mut c_void,
) -> u16
where
    TF: FnMut(SerialServiceEvent) -> u16,
{
    let wrapper: &mut _ = unsafe { &mut *(data as *mut CallbackWrapper<TF>) };
    (wrapper.callback)(status)
}

#[allow(unused)]
pub unsafe extern "C" fn bt_status_changed_callback<TF>(status: BtStatus, data: *mut c_void)
where
    TF: FnMut(BtStatus),
{
    let wrapper: &mut _ = unsafe { &mut *(data as *mut CallbackWrapper<TF>) };
    (wrapper.callback)(status)
}
