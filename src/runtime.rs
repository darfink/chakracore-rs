use error::*;
use chakra_sys::*;

/// An isolated instance of the `ChakraCore` engine.
pub struct Runtime(JsRuntimeHandle);

impl Runtime {
    /// Creates a new runtime.
    pub fn new() -> Result<Runtime> {
        let mut handle = JsRuntimeHandle::new();
        jstry!(unsafe { JsCreateRuntime(JsRuntimeAttributeNone, None, &mut handle) });
        Ok(Runtime(handle))
    }

    /// Sets the runtime's memory limitation.
    pub fn set_memory_limit(&self, limit: usize) -> Result<()> {
        jstry!(unsafe { JsSetRuntimeMemoryLimit(self.as_raw(), limit) });
        Ok(())
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
