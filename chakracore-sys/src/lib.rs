#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
extern crate libc;

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

#[cfg(all(unix, not(feature = "static")))]
pub mod ffi {
    use std::ptr;
    use std::sync::{Once, ONCE_INIT};
    use libc::{c_int, c_void};

    #[link(name = "ChakraCore")]
    extern "system" {
        fn DllMain(instance: *mut c_void, reason: usize, reserved: *mut c_void) -> c_int;
    }

    static START: Once = ONCE_INIT;

    /// This must be called once on Unix when using a shared library.
    pub fn initialize() {
        START.call_once(|| unsafe {
            // This is required on Unix platforms when using a shared library,
            // because ChakraCore depends on `DllMain`, which is not called by
            // default on non-windows platforms.
            DllMain(ptr::null_mut(), 1, ptr::null_mut());
            DllMain(ptr::null_mut(), 2, ptr::null_mut());
        });
    }
}

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
        #[cfg(all(unix, not(feature = "static")))]
        ffi::initialize();

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
            js!(JsCreateStringUtf8(name.as_ptr(), name.len(), &mut name_value));

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
            let mut buffer = vec![0; 100];
            js!(JsCopyStringUtf8(result_as_string,
                                 buffer.as_mut_ptr(),
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
