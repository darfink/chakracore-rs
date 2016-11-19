use std::ptr;
use chakra_sys::*;
use error::*;
use context::ContextGuard;

/// Type for `JsCreateStringUtf8` & `JsCreatePropertyIdUtf8`
pub type StringCallback =
    unsafe extern "system" fn(JsRef, *mut u8, usize, *mut usize) -> JsErrorCode;

/// This is dangerous because it may require an active context.
pub fn to_string_impl(reference: JsRef,
                      callback: StringCallback) -> Result<String> {
    let mut size = 0;
    unsafe {
        // Retrieve how large the string representation is
        jstry!(callback(reference, ptr::null_mut(), 0, &mut size));

        // Allocate an appropriate buffer and retrieve the string
        let mut buffer = vec![0; size];
        jstry!(callback(reference, buffer.as_mut_ptr(), buffer.len(), ptr::null_mut()));

        // Assume the result is valid UTF-8
        Ok(String::from_utf8_unchecked(buffer))
    }
}

pub fn handle_exception(_guard: &ContextGuard, code: JsErrorCode) -> Result<()> {
    match code {
        JsErrorCode::NoError => return Ok(()),
        JsErrorCode::ScriptException => {
            // TODO: Use an exception with stack trace.
            let mut reference = JsValueRef::new();
            jsassert!(unsafe { JsGetAndClearException(&mut reference) });
        },
        _ => (),
    }

    Err(format!("ChakraCore call failed with {:?}", code).into())
}
