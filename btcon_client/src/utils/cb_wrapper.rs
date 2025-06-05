use alloc::boxed::Box;

#[allow(dead_code)]
pub struct CallbackWrapper<'a, F, Ctx> {
    pub callback: &'a mut F,
    pub context: &'a mut Ctx,
}

impl<'a, F, Ctx> CallbackWrapper<'a, F, Ctx> {
    #[allow(unused)]
    pub fn new(callback: &'a mut F, context: &'a mut Ctx) -> *mut Self {
        let boxed = Box::new(CallbackWrapper { callback, context });
        Box::into_raw(boxed)
    }
}
