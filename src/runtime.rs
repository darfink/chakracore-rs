use error::*;
use jsrt_sys::*;

/// An isolated instance of the `JSRT`.
pub struct Runtime(JsRuntimeHandle);

// TODO: Determine whether JsAddRef & JsRelease should be used
// TODO: Add support for promises and more using Builder pattern
impl Runtime {
    /// Creates a new runtime.
    pub fn new() -> Result<Runtime> {
        let mut handle = JsRuntimeHandle::new();
        jstry!(unsafe { JsCreateRuntime(JsRuntimeAttributeNone, None, &mut handle) });
        Ok(Runtime(handle))
    }

    /// Sets the runtime's memory limit.
    pub fn set_memory_limit(&self, limit: usize) -> Result<()> {
        jstry!(unsafe { JsSetRuntimeMemoryLimit(self.as_raw(), limit) });
        Ok(())
    }

    /// Performs a full garbage collection.
    pub fn collect(&self) -> Result<()> {
        jstry!(unsafe { JsCollectGarbage(self.as_raw()) });
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
            jsassert!(JsDisposeRuntime(self.0));
        }
    }
}
