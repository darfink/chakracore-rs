use chakra_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;

#[derive(Clone, Debug)]
pub struct Number(JsValueRef);

impl Number {
    /// Creates a new number.
    pub fn new(_guard: &ContextGuard, number: i32) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            assert_eq!(JsIntToNumber(number, &mut value), JsErrorCode::NoError);
            Number::from_raw(value)
        }
    }

    /// Creates a new number from a double.
    pub fn from_double(_guard: &ContextGuard, number: f64) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            assert_eq!(JsDoubleToNumber(number, &mut value), JsErrorCode::NoError);
            Number::from_raw(value)
        }
    }

    /// Creates a number from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Number(reference)
    }

    /// Converts a JavaScript number to a double.
    pub fn to_double(&self) -> Result<f64> {
        let mut double = 0f64;
        unsafe {
            jstry!(JsNumberToDouble(self.as_raw(), &mut double));
            Ok(double)
        }
    }

    /// Converts a JavaScript number to an integral.
    pub fn to_integer(&self) -> Result<i32> {
        let mut integer = 0;
        unsafe {
            jstry!(JsNumberToInt(self.as_raw(), &mut integer));
            Ok(integer)
        }
    }
}

inherit!(Number, Value);
