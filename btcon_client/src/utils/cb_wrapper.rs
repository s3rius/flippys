use alloc::boxed::Box;

#[allow(dead_code)]
pub struct CallbackWrapper<'a, F> {
    pub callback: &'a mut F,
}

impl<'a, F> CallbackWrapper<'a, F> {
    #[allow(unused)]
    pub fn new(callback: &'a mut F) -> *mut Self {
        let boxed = Box::new(CallbackWrapper { callback });
        Box::into_raw(boxed)
    }
}
