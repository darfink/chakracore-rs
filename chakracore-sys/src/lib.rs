#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
extern crate libc;

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::str;
    use super::*;

    macro_rules! js {
        ($e: expr) => {
            let result = $e;
            if result != JsErrorCode::NoError {
                panic!("JavaScript failed error: {:?}", result);
            }
        }
    }

    #[test]
    fn it_works() {
        unsafe {
            let mut runtime = JsRuntimeHandle::new();
            js!(JsCreateRuntime(JsRuntimeAttributeNone, None, &mut runtime));

            // Create an execution context.
            let mut context = JsContextRef::new();
            js!(JsCreateContext(runtime, &mut context));

            // Now set the current execution context.
            js!(JsSetCurrentContext(context));

            let mut script = String::from("5 + 5");
            let vector = script.as_mut_vec();

            let mut script_buffer = JsValueRef::new();
            js!(JsCreateExternalArrayBuffer(vector.as_mut_ptr() as *mut _,
                                            vector.len() as usize as _,
                                            None,
                                            ptr::null_mut(),
                                            &mut script_buffer));

            let name = "test";
            let mut name_value = JsValueRef::new();
            js!(JsCreateString(name.as_ptr() as *const libc::c_char, name.len(), &mut name_value));

            // Run the script.
            let mut result = JsValueRef::new();
            let source_context = 1;
            js!(JsRun(script_buffer,
                      source_context,
                      name_value,
                      JsParseScriptAttributeNone,
                      &mut result));

            // Convert your script result to String in JavaScript; redundant if your
            // script returns a String
            let mut result_as_string = JsValueRef::new();
            js!(JsConvertValueToString(result, &mut result_as_string));

            // Project script result back to Rust
            let mut size = 0;
            let mut buffer: Vec<u8> = vec![0; 100];
            js!(JsCopyString(result_as_string,
                             buffer.as_mut_ptr() as *mut libc::c_char,
                             buffer.len(),
                             &mut size));
            buffer.truncate(size);

            println!("Output: {}", str::from_utf8_unchecked(&buffer));

            // Dispose runtime
            js!(JsSetCurrentContext(JsValueRef::new()));
            js!(JsDisposeRuntime(runtime));
        }
    }
}
