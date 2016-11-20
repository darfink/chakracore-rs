use jsrt_sys::*;
use context::ContextGuard;
use super::Value;

/// A JavaScript number.
#[derive(Clone, Debug)]
pub struct Number(JsValueRef);

impl Number {
    /// Creates a new number.
    pub fn new(_guard: &ContextGuard, number: i32) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsIntToNumber(number, &mut value));
            Number::from_raw(value)
        }
    }

    /// Creates a new number from a double.
    pub fn from_double(_guard: &ContextGuard, number: f64) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsDoubleToNumber(number, &mut value));
            Number::from_raw(value)
        }
    }

    /// Creates a number from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Number(reference)
    }

    /// Converts a JavaScript number to a double.
    pub fn value_double(&self) -> f64 {
        let mut double = 0f64;
        unsafe {
            jsassert!(JsNumberToDouble(self.as_raw(), &mut double));
            double
        }
    }

    /// Converts a JavaScript number to an integral.
    pub fn value(&self) -> i32 {
        let mut integer = 0;
        unsafe {
            jsassert!(JsNumberToInt(self.as_raw(), &mut integer));
            integer
        }
    }
}

inherit!(Number, Value);
