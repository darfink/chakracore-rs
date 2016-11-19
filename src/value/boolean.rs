use chakra_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;

#[derive(Clone, Debug)]
pub struct Boolean(JsValueRef);

impl Boolean {
    /// Creates a new boolean.
    pub fn new(_guard: &ContextGuard, boolean: bool) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            assert_eq!(JsBoolToBoolean(boolean, &mut value), JsErrorCode::NoError);
            Boolean::from_raw(value)
        }
    }

    /// Creates a number from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Boolean(reference)
    }

    /// Converts a JavaScript boolean to a bool.
    pub fn to_bool(&self) -> Result<bool> {
        let mut boolean = false;
        unsafe {
            jstry!(JsBooleanToBool(self.as_raw(), &mut boolean));
            Ok(boolean)
        }
    }
}

inherit!(Boolean, Value);
