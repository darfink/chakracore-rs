use error::*;
use chakra_sys::*;

pub struct Runtime(JsRuntimeHandle);

impl Runtime {
    /// Creates a new runtime.
    pub fn new() -> Result<Runtime> {
        let mut handle = JsRuntimeHandle::new();
        jstry!(unsafe { JsCreateRuntime(JsRuntimeAttributeNone, None, &mut handle) });
        Ok(Runtime(handle))
    }

    /// Returns the underlying raw pointer behind this runtime.
    pub fn as_raw(&self) -> JsRuntimeHandle {
        self.0
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(JsDisposeRuntime(self.0), JsErrorCode::NoError);
        }
    }
}
