//! Javascript values that user code can interact with.
use context::ContextGuard;
use jsrt_sys::*;

pub use self::array::*;
pub use self::boolean::Boolean;
pub use self::function::Function;
pub use self::number::Number;
pub use self::object::Object;
pub use self::string::String;
pub use self::value::Value;

// Modules
pub mod function;
pub mod object;
mod array;
mod boolean;
mod number;
mod string;
mod value;

/// Creates a false value.
pub fn false_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetFalseValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates a false value.
pub fn true_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetTrueValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates a null value.
pub fn null(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetNullValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates an undefined value.
pub fn undefined(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetUndefinedValue(&mut value));
        Value::from_raw(value)
    }
}
