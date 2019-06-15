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
//! let result = js::script::eval(&guard, "10 + 10").unwrap();
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
//! let add = js::script::parse(&guard, "10 + 10").unwrap();
//! let result = add.call(&guard, &[]).unwrap();
//! assert_eq!(result.to_integer(&guard), 20);
//! ```
use chakracore_sys::*;
use error::*;
use context::ContextGuard;
use util::jstry;
use value;

/// Evaluates code directly.
pub fn eval(guard: &ContextGuard, code: &str) -> Result<value::Value> {
    eval_with_name(guard, "", code)
}

/// Evaluates code and associates it with a name.
pub fn eval_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Value> {
    process_code(guard, name, code, CodeAction::Execute)
}

/// Parses code and returns it as a function.
pub fn parse(guard: &ContextGuard, code: &str) -> Result<value::Function> {
    parse_with_name(guard, "", code)
}

/// Parses code and associates it with a name, returns it as a function.
pub fn parse_with_name(guard: &ContextGuard, name: &str, code: &str) -> Result<value::Function> {
    process_code(guard, name, code, CodeAction::Parse)
        .map(|value| value
            .into_function()
            .expect("converting parsing result to function"))
}

/// Used for processing code.
#[derive(Copy, Clone, Debug)]
enum CodeAction {
    Execute,
    Parse,
}

// TODO: Should `JavascriptExternalArrayBuffer` be supported? Lifetime issues.
/// Either parses or executes a script.
fn process_code(guard: &ContextGuard, name: &str, code: &str, action: CodeAction) -> Result<value::Value> {
    let name = value::String::new(guard, name);
    let buffer = value::String::new(guard, code);

    let api = match action {
        CodeAction::Execute => JsRun,
        CodeAction::Parse => JsParse,
    };

    unsafe {
        let mut result = JsValueRef::new();
        jstry(api(buffer.as_raw(),
                  generate_source_context(),
                  name.as_raw(),
                  JsParseScriptAttributeNone,
                  &mut result))
            .map(|_| value::Value::from_raw(result))
    }
}

/// Generates a new source context identifier.
fn generate_source_context() -> JsSourceContext {
    // TODO: handle source context identifier
    JsSourceContext::max_value()
}

#[cfg(test)]
mod tests {
    use {test, Error, script};

    #[test]
    fn execute_exception() {
        test::run_with_context(|guard| {
            let error = script::eval(guard, "null[0] = 3;").unwrap_err();
            let result = script::eval(guard, "5 + 5").unwrap();

            assert_matches!(error, Error::ScriptException(_));
            assert_eq!(result.to_integer(guard), 10);
        });
    }

    #[test]
    fn compile_exception() {
        test::run_with_context(|guard| {
            let error = script::eval(guard, "err)").unwrap_err();
            let result = script::eval(guard, "5 + 5").unwrap();

            assert_eq!(result.to_integer(guard), 10);
            assert_matches!(error, Error::ScriptCompilation(_));
        });
    }

    #[test]
    fn parse_script() {
        test::run_with_context(|guard| {
            let func = script::parse(guard, "new Number(10)").unwrap();
            let result = func.call(guard, &[]).unwrap();
            assert_eq!(result.to_integer(guard), 10);
        });
    }
}
