//! Javascript values that user code can interact with.
//!
//! The `Value` object acts as the base type for all others using dereference.
//! Therefore all types have access to the `Value` type's methods and traits.
//! Implicit traits such as `Debug` must be accessed by dereferencing. For
//! example: the `Function` inherits `Object` which then inherits `Value`. To
//! print a function using the debug trait, you need to dereference twice.
//!
//! ```norun
//! println("Output: {:?}", **function_value);
//! ```
//!
//! Please note that the `Debug` trait should be carefully used. It relies
//! implicitly on an active context, and that it's the same that the value was
//! created with. Therefore it's mostly implemented to ease debugging. Prefer
//! `to_string` in stable code.
//!
//! All created values are tied to a specific context. Because of this a
//! `ContextGuard` is required when creating new values, and you cannot pass
//! values to different scripts.
use context::ContextGuard;
use chakracore_sys::*;

pub use self::array::*;
pub use self::boolean::Boolean;
pub use self::error::Error;
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
mod error;
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
