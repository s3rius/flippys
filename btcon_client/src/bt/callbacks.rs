use core::ffi::c_void;

use flipperzero_sys::{BtStatus, SerialServiceEvent};

use crate::utils::CallbackWrapper;

#[allow(unused)]
pub unsafe extern "C" fn ble_svc_serial_callback<'b, TF, TCtx>(
    status: SerialServiceEvent,
    data: *mut c_void,
) -> u16
where
    TF: FnMut(SerialServiceEvent, &mut TCtx) -> u16 + 'b,
    TCtx: 'b,
{
    let wrapper: &mut _ = unsafe { &mut *(data as *mut CallbackWrapper<'b, TF, TCtx>) };
    (wrapper.callback)(status, &mut wrapper.context)
}

#[allow(unused)]
pub unsafe extern "C" fn bt_status_changed_callback<'b, TF, TCtx>(
    status: BtStatus,
    data: *mut c_void,
) where
    TF: FnMut(BtStatus, &'b mut TCtx) + 'b,
    TCtx: 'b,
{
    let wrapper: &mut _ = unsafe { &mut *(data as *mut CallbackWrapper<'b, TF, TCtx>) };
    (wrapper.callback)(status, &mut wrapper.context)
}
