use std::ptr;
use chakracore_sys::*;
use error::*;
use context::{Context, ContextGuard};
use value;

/// Type for `JsCreateString` & `JsCreatePropertyId`
pub type StringCall = unsafe extern "system" fn(JsRef, *mut i8, usize, *mut usize) -> JsErrorCode;

/// This is dangerous because it may require an active context.
pub fn to_string_impl(reference: JsRef, callback: StringCall) -> Result<String> {
    let mut size = 0;
    unsafe {
        // Determine how large the string representation is
        jstry!(callback(reference, ptr::null_mut(), 0, &mut size));

        // Allocate an appropriate buffer and retrieve the string
        let mut buffer: Vec<u8> = vec![0; size];
        jstry!(callback(reference,
                        buffer.as_mut_ptr() as _,
                        buffer.len(),
                        ptr::null_mut()));

        // Assume the result is valid UTF-8
        Ok(String::from_utf8_unchecked(buffer))
    }
}

/// Decrements a reference counter and asserts its value.
pub fn release_reference(reference: JsRef) {
    let mut count = 0;
    jsassert!(unsafe { JsRelease(reference, &mut count) });
    debug_assert!(count < ::libc::c_uint::max_value());
}

/// Returns a script result as a `Function`.
///
/// This is useful for functionality that the underlying JSRT API does not
/// provide, such as JSON methods, or `RegExp` constructor.
pub fn jsfunc(guard: &ContextGuard, function: &str) -> Option<value::Function> {
    ::script::eval(guard, function).ok().and_then(|val| val.into_function())
}

/// Converts a JSRT error code to a result.
pub fn jstry(code: JsErrorCode) -> Result<()> {
    match code {
        JsErrorCode::NoError => Ok(()),
        JsErrorCode::ScriptException | JsErrorCode::ScriptCompile => {
            Context::exec_with_current(|guard| {
                // TODO: Use an exception with stack trace.
                let exception = get_and_clear_exception(guard);
                let message = exception.to_string(guard);

                Err(if code == JsErrorCode::ScriptException {
                    Error::ScriptException(message)
                } else {
                    Error::ScriptCompilation(message)
                })
            }).expect("active context in result handler")
        },
        error @ _ => Err(Error::JsrtCall(error)),
    }
}

/// Retrieves and clears any exception thrown during compilation or execution.
///
/// The runtime is set to a disabled state whenever an exception is thrown.
fn get_and_clear_exception(_guard: &ContextGuard) -> value::Value {
    let mut exception = JsValueRef::new();
    unsafe {
        jsassert!(JsGetAndClearException(&mut exception));
        value::Value::from_raw(exception)
    }
}
