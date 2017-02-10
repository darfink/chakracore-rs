//! Functionality for executing and parsing JavaScript.
//!
//! The simple `eval` function should cover most needs. It evalutes the supplied
//! code directly and returns the script's value.
//!
//! ```rust
//! # use chakracore as js;
//! # let runtime = js::Runtime::new().unwrap();
//! # let context = js::Context::new(&runtime).unwrap();
//! # let guard = context.make_current().unwrap();
//! let result = js::script::eval(&guard, "(10 + 10)").unwrap();
//! assert_eq!(result.to_integer(&guard), 20);
//! ```
//!
//! Another option is to parse the source code and execute it at a later time
//! with a function. This is done using the `parse` function:
//!
//! ```rust
//! # use chakracore as js;
//! # let runtime = js::Runtime::new().unwrap();
//! # let context = js::Context::new(&runtime).unwrap();
//! # let guard = context.make_current().unwrap();
//! let add = js::script::parse(&guard, "(10 + 10)").unwrap();
//! let result = add.call(&guard, &[]).unwrap();
//! assert_eq!(result.to_integer(&guard), 20);
//! ```
use std::slice;
use chakracore_sys::*;
use error::*;
use context::ContextGuard;
use value;
use util;

/// Evaluates code directly.
pub fn eval(guard: &ContextGuard, code: &str) -> Result<value::Value> {
    eval_with_name(guard, "", code)
}

/// Evaluates code and associates it with a name.
pub fn eval_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Value> {
    let (code, result) = process_code(guard, name, code, CodeAction::Execute);
    unsafe {
        util::handle_exception(guard, code).map(|_| value::Value::from_raw(result))
    }
}

/// Parses code and returns it as a function.
pub fn parse(guard: &ContextGuard, code: &str) -> Result<value::Function> {
    parse_with_name(guard, "", code)
}

/// Parses code and associates it with a name, returns it as a function.
pub fn parse_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Function> {
    let (code, result) = process_code(guard, name, code, CodeAction::Parse);
    jstry!(code);
    Ok(unsafe { value::Function::from_raw(result) })
}

/// A serialized JavaScript source, in a runtime-independent format.
pub struct Buffer(value::ArrayBuffer);

impl Buffer {
    /// Serializes code and returns it as a buffer
    pub fn new(guard: &ContextGuard, code: &str) -> Result<Self> {
        // The size of the serialized code is often much large than the source
        let code_source = create_code_buffer(guard, code);
        Ok(unsafe {
        let mut result = JsValueRef::new();
            jstry!(JsSerialize(code_source.as_raw(),
                               &mut result,
                               JsParseScriptAttributeNone));
            Buffer(value::ArrayBuffer::from_raw(result))
        })
    }

    /// Parses the serialized source and returns it as a function.
    pub fn parse(&mut self, guard: &ContextGuard, name: &str) -> Result<value::Function> {
        let name = value::String::new(guard, name);
        let mut result = JsValueRef::new();
        unsafe {
            // TODO: The api information is invalid, callback cannot be null
            jstry!(JsParseSerialized(self.0.as_raw(),
                                     None,
                                     generate_source_context(),
                                     name.as_raw(),
                                     &mut result));
            Ok(value::Function::from_raw(result))
        }
    }

    /// Runs the serialized code in a specificed context.
    pub fn run(&mut self, guard: &ContextGuard) -> Result<value::Value> {
        self.run_with_name(guard, "")
    }

    /// Runs the serialized code in a specificed context, and associates it with a name.
    pub fn run_with_name(&mut self, guard: &ContextGuard, name: &str) -> Result<value::Value> {
        let name = value::String::new(guard, name);
        let mut result = JsValueRef::new();
        unsafe {
            // TODO: Analyze why a callback is required and if it should be used
            let code = JsRunSerialized(self.0.as_raw(),
                                       None,
                                       generate_source_context(),
                                       name.as_raw(),
                                       &mut result);
            util::handle_exception(guard, code).map(|_| value::Value::from_raw(result))
        }
    }

    /// Consumes the buffer and returns its internal byte representation.
    pub fn into_buffer(self) -> value::ArrayBuffer {
        self.0
    }
}

/// Used for processing code.
#[derive(Copy, Clone, Debug)]
enum CodeAction {
    Execute,
    Parse,
}

/// Either parses or executes a script.
fn process_code(guard: &ContextGuard, name: &str, code: &str, action: CodeAction) -> (JsErrorCode, JsValueRef) {
    let name = value::String::new(guard, name);
    let buffer = create_code_buffer(guard, code);

    let api = match action {
        CodeAction::Execute => JsRun,
        CodeAction::Parse => JsParse,
    };

    unsafe {
        let mut result = JsValueRef::new();
        let code = api(buffer.as_raw(),
                        generate_source_context(),
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
fn generate_source_context() -> JsSourceContext {
    // TODO: handle source context identifier
    1
}
