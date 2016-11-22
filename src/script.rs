use std::{slice, ptr};
use chakracore_sys::*;
use error::*;
use context::ContextGuard;
use value;
use util;

/// A compiled JavaScript source, in a runtime-independent format.
pub struct Script(Vec<u8>);

// TODO: handle parsing of serialized scripts
impl Script {
    /// Creates a script handle from serialized code.
    pub fn from_serialized_code(data: Vec<u8>) -> Script {
        Script(data)
    }

    /// Evaluates code in the specified context.
    pub fn eval(guard: &ContextGuard, code: &str) -> Result<value::Value> {
        Self::eval_with_name(guard, "", code)
    }

    /// Evalutes code and associates it with a name.
    pub fn eval_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Value> {
        let (code, result) = Self::process_code(guard, name, code, CodeAction::Execute);
        unsafe {
            util::handle_exception(guard, code).map(|_| value::Value::from_raw(result))
        }
    }

    /// Parses code an returns it as a function.
    pub fn parse(guard: &ContextGuard, code: &str) -> Result<value::Function> {
        Self::parse_with_name(guard, "", code)
    }

    /// Parses code, associates it with a name, and returns it as a function.
    pub fn parse_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Function> {
        let (code, result) = Self::process_code(guard, name, code, CodeAction::Parse);
        jstry!(code);
        Ok(unsafe { value::Function::from_raw(result) })
    }

    /// Parse code and stores it in a runtime-independant format.
    pub fn serialize(guard: &ContextGuard, code: &str) -> Result<Script> {
        // The size of the serialized code is often much large than the source
        let code_source = Self::create_code_buffer(guard, code);
        let mut serialized_size = 0;
        Ok(unsafe {
            jstry!(JsSerialize(code_source.as_raw(),
                            ptr::null_mut(),
                            &mut serialized_size,
                            JsParseScriptAttributeNone));
            let mut serialized_code = vec![0; serialized_size as usize];
            jstry!(JsSerialize(code_source.as_raw(),
                            serialized_code.as_mut_ptr(),
                            &mut serialized_size,
                            JsParseScriptAttributeNone));
            Script(serialized_code)
        })
    }

    /// Parses the serialized source and returns it as a function.
    pub fn unserialize(&mut self, guard: &ContextGuard, name: &str) -> Result<value::Function> {
        Self::parse_serialized(guard, name, &mut self.0)
    }

    /// Runs the serialized code in a specificed context.
    pub fn run(&mut self, guard: &ContextGuard) -> Result<value::Value> {
        self.run_with_name(guard, "")
    }

    /// Runs the serialized code in a specificed context, and associates it with a name.
    pub fn run_with_name(&mut self, guard: &ContextGuard, name: &str) -> Result<value::Value> {
        let name = value::String::from_str(guard, name);
        let mut result = JsValueRef::new();
        unsafe {
            // TODO: Analyze why a callback is required and if it should be used
            let code = JsRunSerialized(self.0.as_mut_ptr(),
                                       None,
                                       Self::generate_source_context(),
                                       name.as_raw(),
                                       &mut result);
            util::handle_exception(guard, code).map(|_| value::Value::from_raw(result))
        }
    }

    /// Consumes the script and returns its internal byte representation.
    pub fn into_serialized_code(self) -> Vec<u8> {
        self.0
    }

    /// Parses a serialized code and returns it as a function.
    fn parse_serialized(guard: &ContextGuard, name: &str, data: &mut Vec<u8>) -> Result<value::Function> {
        let name = value::String::from_str(guard, name);
        let mut result = JsValueRef::new();
        unsafe {
            // TODO: The api information is invalid, callback cannot be null
            jstry!(JsParseSerialized(data.as_mut_ptr(),
                                     None,
                                     Self::generate_source_context(),
                                     name.as_raw(),
                                     &mut result));
            Ok(value::Function::from_raw(result))
        }
    }

    /// Either parses or executes a script.
    fn process_code(guard: &ContextGuard, name: &str, code: &str, action: CodeAction) -> (JsErrorCode, JsValueRef) {
        let name = value::String::from_str(guard, name);
        let buffer = Self::create_code_buffer(guard, code);

        let api = match action {
            CodeAction::Execute => JsRun,
            CodeAction::Parse => JsParse,
        };

        unsafe {
            let mut result = JsValueRef::new();
            let code = api(buffer.as_raw(),
                           Self::generate_source_context(),
                           name.as_raw(),
                           JsParseScriptAttributeNone,
                           &mut result);
            (code, result)
        }
    }

    /// Creates an array buffer from immutable data (JSRT does not modify it internally).
    fn create_code_buffer(guard: &ContextGuard, code: &str) -> value::ArrayBuffer {
        let bytes = code.as_bytes();
        unsafe {
            // It's assumed that the JSRT implementation does not modify the code buffer
            let slice = slice::from_raw_parts_mut(bytes.as_ptr() as *mut u8, bytes.len());
            value::ArrayBuffer::from_slice(guard, slice)
        }
    }

    /// Generates a new source context identifier.
    fn generate_source_context() -> usize {
        // TODO: handle source context identifier
        1
    }
}

/// Used for processing code.
#[derive(Copy, Clone, Debug)]
enum CodeAction {
    Execute,
    Parse,
}
