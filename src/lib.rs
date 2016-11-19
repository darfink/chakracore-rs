#[macro_use]
extern crate error_chain;
extern crate chakra_sys;
extern crate libc;

pub use context::Context;
pub use runtime::Runtime;
pub use script::Script;

#[macro_use]
mod macros;
mod script;
mod runtime;
pub mod context;
pub mod error;
pub mod value;

#[cfg(test)]
mod tests {
    use super::*;

    fn callback(guard: &context::ContextGuard,
                info: &value::function::CallbackInfo)
                -> ::std::result::Result<value::Value, value::Value> {
        println!("Faggot! {:#?}", info);
        Ok(value::Number::new(guard, 120).into())
    }

    #[test]
    fn basic() {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();

        let guard = context.make_current().unwrap();

        // TODO: Wrap input in parantheses?
        let result = Script::run(&guard, "(5 + 5)").unwrap();
        assert_eq!(result.to_integer_convert(&guard).unwrap(), 10);
    }

    #[test]
    fn function() {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();

        let guard = context.make_current().unwrap();
        let function = value::Function::new(&guard, Box::new(callback)).unwrap();

        let result = function.call(&guard, &value::null(&guard), &[
            value::Boolean::new(&guard, true).into(),
            value::String::from_str(&guard, "Test").unwrap().into(),
            value::Number::from_double(&guard, 3.141414).into(),
            value::Number::new(&guard, 10).into(),
        ]).unwrap();

        assert_eq!(result.to_double_convert(&guard).unwrap(), 120f64);
        println!("Result: {:?}", result);
    }
}
