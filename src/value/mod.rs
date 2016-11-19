use context::ContextGuard;
use chakra_sys::*;

pub use self::function::Function;
pub use self::boolean::Boolean;
pub use self::number::Number;
pub use self::object::Object;
pub use self::string::String;
pub use self::value::Value;

// Modules
pub mod function;
pub mod object;
mod boolean;
mod number;
mod string;
mod value;

/// Creates a false value.
pub fn false_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        assert_eq!(JsGetFalseValue(&mut value), JsErrorCode::NoError);
        Value::from_raw(value)
    }
}

/// Creates a false value.
pub fn true_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        assert_eq!(JsGetTrueValue(&mut value), JsErrorCode::NoError);
        Value::from_raw(value)
    }
}

/// Creates a null value.
pub fn null(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        assert_eq!(JsGetNullValue(&mut value), JsErrorCode::NoError);
        Value::from_raw(value)
    }
}

/// Creates an undefined value.
pub fn undefined(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        assert_eq!(JsGetUndefinedValue(&mut value), JsErrorCode::NoError);
        Value::from_raw(value)
    }
}
