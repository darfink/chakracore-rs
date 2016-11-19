use chakra_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;

#[derive(Clone, Debug)]
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
            assert_eq!(JsCreateStringUtf8(string.as_ptr(),
                                          string.len(),
                                          &mut value),
                       JsErrorCode::NoError);
            String(value)
        }
    }

    /// Returns the length of the string.
    pub fn len(&self) -> Result<usize> {
        let mut length = 0;
        jstry!(unsafe { JsGetStringLength(self.as_raw(), &mut length) });
        Ok(length as usize)
    }

    /// Converts a JavaScript string to a native string.
    pub fn to_string(&self) -> Result<::std::string::String> {
        ::util::to_string_impl(self.as_raw(), JsCopyStringUtf8)
    }
}

inherit!(String, Value);
