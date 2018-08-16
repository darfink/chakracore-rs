//! Javascript values that user code can interact with.
//!
//! The [Value](struct.Value.html) object acts as the base type for all others
//! using dereference.  Therefore all types have access to the `Value` type's
//! methods and traits.  Implicit traits such as `Debug`, must be accessed by
//! dereferencing. For example: `Function` inherits `Object`, which in turn
//! inherits `Value`. To print a function using the debug trait, you need to
//! dereference twice.
//!
//! ```norun
//! println("Output: {:?}", **function_value);
//! ```
//!
//! Please note that the `Debug` trait should be carefully used. It relies
//! implicitly on an active context, and that it's the same context the value
//! was created with. Therefore it's mostly implemented to ease debugging.
//! Prefer `to_string` for safety.
//!
//! All created values are tied to a specific context. Because of this a
//! `ContextGuard` is required whenever creating new values, and they should
//! not be passed between different contexts.
use context::ContextGuard;
use chakracore_sys::*;

// TODO: Add typed arrays and buffer view.
pub use self::array::*;
pub use self::boolean::Boolean;
pub use self::error::Error;
pub use self::external::External;
pub use self::function::Function;
pub use self::number::Number;
pub use self::object::Object;
pub use self::promise::Promise;
pub use self::string::String;
pub use self::value::Value;

#[macro_use]
mod macros;

// Modules
pub mod function;
pub mod promise;
mod object;
mod array;
mod boolean;
mod error;
mod external;
mod number;
mod string;
mod value;

/// Creates a `false` value.
pub fn false_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetFalseValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates a `true` value.
pub fn true_(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetTrueValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates a `null` value.
pub fn null(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetNullValue(&mut value));
        Value::from_raw(value)
    }
}

/// Creates an `undefined` value.
pub fn undefined(_guard: &ContextGuard) -> Value {
    let mut value = JsValueRef::new();
    unsafe {
        jsassert!(JsGetUndefinedValue(&mut value));
        Value::from_raw(value)
    }
}

// These are hidden and exists merely for the `is_same` functionality.
struct Null;
struct Undefined;

impl Null { is_same!(Null, "Returns true if the value is `null`"); }
impl Undefined { is_same!(Undefined, "Returns true if the value is `undefined`"); }
