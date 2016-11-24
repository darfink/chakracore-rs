use chakracore_sys::*;
use context::ContextGuard;
use super::{Value, Object};
use Property;

macro_rules! ctor {
    ($name:ident, $errtype:ident, $doc:expr) => {
        #[doc=$doc]
        pub fn $name(guard: &ContextGuard, message: &str) -> Self {
            create_error(guard, message, $errtype)
        }
    };
}

/// A JavaScript error.
#[derive(Clone)]
pub struct Error(JsValueRef);

impl Error {
    ctor!(new, JsCreateError, "Creates a new error.");
    ctor!(range_error, JsCreateRangeError, "Creates a new range error.");
    ctor!(reference_error, JsCreateReferenceError, "Creates a new reference error.");
    ctor!(syntax_error, JsCreateSyntaxError, "Creates a new syntax error.");
    ctor!(type_error, JsCreateTypeError, "Creates a new type error.");
    ctor!(uri_error, JsCreateURIError, "Creates a new URI error.");

    /// Creates an error from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Error(reference)
    }

    /// Returns the error's message.
    pub fn message(&self, guard: &ContextGuard) -> String {
        let property = Property::from_str(guard, "message");
        self.get(guard, &property).to_string(guard)
    }

    is_same!(Error, "Returns true if the value is an `Error`.");
}

/// Function definition for an error call.
type ErrorCall = unsafe extern "system" fn(JsValueRef, *mut JsValueRef) -> JsErrorCode;

/// Creates an error object from a specified API.
fn create_error(guard: &ContextGuard, message: &str, api: ErrorCall) -> Error {
    let message = super::String::from_str(guard, message);
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(api(message.as_raw(), &mut value));
        Error::from_raw(value)
    }
}

inherit!(Error, Object);
subtype!(Error, Value);
