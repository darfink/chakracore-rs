use chakra_sys::*;
use context::ContextGuard;
use super::Value;

/// A JavaScript boolean.
#[derive(Clone, Debug)]
pub struct Boolean(JsValueRef);

impl Boolean {
    /// Creates a new boolean.
    pub fn new(_guard: &ContextGuard, boolean: bool) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsBoolToBoolean(boolean, &mut value));
            Boolean::from_raw(value)
        }
    }

    /// Creates a number from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Boolean(reference)
    }

    /// Converts a JavaScript boolean to a bool.
    pub fn value(&self) -> bool {
        let mut boolean = false;
        jsassert!(unsafe { JsBooleanToBool(self.as_raw(), &mut boolean) });
        boolean
    }
}

inherit!(Boolean, Value);
