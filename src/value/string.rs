use chakracore_sys::*;
use context::ContextGuard;
use super::Value;

/// A JavaScript string.
pub struct String(JsValueRef);

impl String {
    /// Creates a string value from a native string.
    pub fn new(_guard: &ContextGuard, string: &str) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateString(string.as_ptr() as _, string.len(), &mut value));
            Self::from_raw(value)
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
        ::util::to_string_impl(self.as_raw(), JsCopyString).unwrap()
    }

    is_same!(String, "Returns true if the value is a `String`.");
}

reference!(String);
inherit!(String, Value);
