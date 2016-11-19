#[macro_use]
extern crate error_chain;
extern crate chakra_sys;
extern crate libc;

pub use context::Context;
pub use runtime::Runtime;
pub use script::Script;
pub use property::PropertyId;

#[macro_use]
mod macros;
mod property;
mod script;
mod runtime;
mod util;
pub mod context;
pub mod error;
pub mod value;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() -> (Runtime, Context) {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();
        (runtime, context)
    }

    fn callback(guard: &context::ContextGuard,
                _info: &value::function::CallbackInfo)
                -> ::std::result::Result<value::Value, value::Value> {
        Ok(value::Number::new(guard, 120).into())
    }

    #[test]
    fn basic_runtime() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        // TODO: Wrap input in parantheses?
        let result = Script::run(&guard, "(5 + 5)").unwrap();
        assert_eq!(result.to_integer_convert(&guard).unwrap(), 10);
    }

    #[test]
    fn function_call() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let function = value::Function::new(&guard, Box::new(callback)).unwrap();

        let result = function.call(&guard, &value::null(&guard), &[
            value::Boolean::new(&guard, true).into(),
            value::String::from_str(&guard, "Test").into(),
            value::Number::from_double(&guard, 3.141414).into(),
            value::Number::new(&guard, 10).into(),
        ]).unwrap();

        assert_eq!(result.to_integer_convert(&guard).unwrap(), 120);
        assert_eq!(result.to_double_convert(&guard).unwrap(), 120f64);
    }

    #[test]
    fn global_context() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let global = guard.global().unwrap();

        let dirname = PropertyId::from_str(&guard, "__dirname");
        global.set(&guard, &dirname, &value::String::from_str(&guard, "/pwd/n00b/")).unwrap();
        global.set_index(&guard, 0, &value::String::from_str(&guard, "test")).unwrap();

        let _result = Script::run(&guard, "__dirname").unwrap();
        let result = Script::run(&guard, "this[0]").unwrap();
        println!("Out: {:?}", result);
    }
}
