use chakracore_sys::*;
use context::ContextGuard;
use super::Value;

/// A JavaScript string.
#[derive(Clone)]
pub struct String(JsValueRef);

impl String {
    /// Creates a new empty string.
    pub fn new(guard: &ContextGuard) -> Self {
        Self::from_str(guard, "")
    }

    /// Creates a string from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        String(reference)
    }

    /// Creates a string value from a native string.
    pub fn from_str(_guard: &ContextGuard, string: &str) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateStringUtf8(string.as_ptr(), string.len(), &mut value));
            String(value)
        }
    }

    /// Returns the length of the string.
    pub fn len(&self) -> usize {
        let mut length = 0;
        jsassert!(unsafe { JsGetStringLength(self.as_raw(), &mut length) });
        length as usize
    }

    /// Converts a JavaScript string to a native string.
    pub fn value(&self) -> ::std::string::String {
        ::util::to_string_impl(self.as_raw(), JsCopyStringUtf8).unwrap()
    }

    is_same!(String, "Returns true if the value is an `String`.");
}

inherit!(String, Value);
