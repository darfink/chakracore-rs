use std::ptr;
use chakra_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;

#[derive(Clone, Debug)]
pub struct String(JsValueRef);

impl String {
    /// Creates a new empty string.
    pub fn new(guard: &ContextGuard) -> Result<Self> {
        Self::from_str(guard, "")
    }

    /// Creates a string from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        String(reference)
    }

    /// Creates a string value from a native string.
    pub fn from_str(_guard: &ContextGuard, string: &str) -> Result<Self> {
        let mut value = JsValueRef::new();
        jstry!(unsafe { JsCreateStringUtf8(string.as_ptr(), string.len(), &mut value) });
        Ok(String(value))
    }

    /// Returns the length of the string.
    pub fn len(&self) -> Result<usize> {
        let mut length = 0;
        jstry!(unsafe { JsGetStringLength(self.as_raw(), &mut length) });
        Ok(length as usize)
    }

    /// Converts a JavaScript string to a native string.
    pub fn to_string(&self) -> Result<::std::string::String> {
        let mut size = 0;
        unsafe {
            // Retrieve how large the string representation is
            jstry!(JsCopyStringUtf8(self.as_raw(), ptr::null_mut(), 0, &mut size));

            // Allocate an appropriate buffer and retrieve the string
            let mut buffer = vec![0; size];
            jstry!(JsCopyStringUtf8(self.as_raw(), buffer.as_mut_ptr(), buffer.len(), ptr::null_mut()));

            // Assume the result is valid UTF-8
            Ok(::std::string::String::from_utf8_unchecked(buffer))
        }
    }
}

inherit!(String, Value);
