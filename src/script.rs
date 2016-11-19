use std::slice;
use chakra_sys::*;
use error::*;
use context::ContextGuard;
use value;

pub struct Script;

// TODO: serialized and parsed scripts
impl Script {
    /// Runs the script without an associated name
    pub fn run(guard: &ContextGuard, code: &str) -> Result<value::Value> {
        Self::run_with_name(guard, "", code)
    }

    /// Runs the script in the specified context.
    pub fn run_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Value> {
        let bytes = code.as_bytes();

        let name = value::String::from_str(guard, name);
        let buffer = unsafe {
            // It's assumed that ChakraCore engine does not modify the code buffer.
            let slice = slice::from_raw_parts_mut(bytes.as_ptr() as *mut u8, bytes.len());
            value::Value::from_external_slice(guard, slice)
        }?;

        let mut identifier = 1;
        let mut result = JsValueRef::new();

        unsafe {
            // TODO: handle exceptions!
            jstry!(JsRun(buffer.as_raw(),
                         &mut identifier,
                         name.as_raw(),
                         JsParseScriptAttributeNone,
                         &mut result));
            Ok(value::Value::from_raw(result))
        }
    }
}
