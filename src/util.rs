use std::ptr;
use chakracore_sys::*;
use error::*;
use context::ContextGuard;
use value;

/// Type for `JsCreateStringUtf8` & `JsCreatePropertyIdUtf8`
pub type StringCall = unsafe extern "system" fn(JsRef, *mut u8, usize, *mut usize) -> JsErrorCode;

/// This is dangerous because it may require an active context.
pub fn to_string_impl(reference: JsRef, callback: StringCall) -> Result<String> {
    let mut size = 0;
    unsafe {
        // Determine how large the string representation is
        jstry!(callback(reference, ptr::null_mut(), 0, &mut size));

        // Allocate an appropriate buffer and retrieve the string
        let mut buffer = vec![0; size];
        jstry!(callback(reference,
                        buffer.as_mut_ptr(),
                        buffer.len(),
                        ptr::null_mut()));

        // Assume the result is valid UTF-8
        Ok(String::from_utf8_unchecked(buffer))
    }
}

/// The runtime is set to a disabled state whenever an exception is thrown.
pub fn handle_exception(guard: &ContextGuard, code: JsErrorCode) -> Result<()> {
    match code {
        JsErrorCode::NoError => Ok(()),
        JsErrorCode::ScriptException => {
            // TODO: Use an exception with stack trace.
            let exception = get_and_clear_exception(guard);
            Err(ErrorKind::ScriptException(exception.to_string(guard)).into())
        },
        JsErrorCode::ScriptCompile => {
            let exception = get_and_clear_exception(guard);
            Err(ErrorKind::ScriptCompile(exception.to_string(guard)).into())
        },
        _ => Err(format!("JSRT call failed with {:?}", code).into()),
    }
}

fn get_and_clear_exception(_: &ContextGuard) -> value::Value {
    let mut reference = JsValueRef::new();
    jsassert!(unsafe { JsGetAndClearException(&mut reference) });

    let exception = unsafe { value::Value::from_raw(reference) };
    exception
}
