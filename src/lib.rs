#[macro_use]
extern crate error_chain;
extern crate chakracore_sys;
extern crate libc;

pub use context::Context;
pub use runtime::Runtime;
pub use property::Property;

#[macro_use]
mod macros;
mod property;
mod util;
pub mod runtime;
pub mod context;
pub mod error;
pub mod script;
pub mod value;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() -> (Runtime, Context) {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();
        (runtime, context)
    }

    #[test]
    fn basic_runtime() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        // TODO: Wrap input in parantheses?
        let result = script::eval(&guard, "(5 + 5)").unwrap();
        assert_eq!(result.to_integer(&guard), 10);
    }

    #[test]
    fn basic_exception() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        let error = script::eval(&guard, "null[0] = 3;");
        let result = script::eval(&guard, "(5 + 5)").unwrap();

        assert_eq!(result.to_integer(&guard), 10);
        match error.unwrap_err().kind() {
            &error::ErrorKind::ScriptException(_) => assert!(true),
            _ => assert!(false),
        };
    }

    #[test]
    fn global_context() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();

        let global = guard.global();
        let dirname = Property::from_str(&guard, "__dirname");

        global.set(&guard, &dirname, &value::String::from_str(&guard, "FooBar"));
        global.set_index(&guard, 2, &value::Number::new(&guard, 1337));

        let result1 = script::eval(&guard, "__dirname").unwrap();
        let result2 = script::eval(&guard, "this[2]").unwrap();

        assert_eq!(result1.to_string(&guard), "FooBar");
        assert_eq!(result2.to_integer(&guard), 1337);
    }

    #[test]
    fn function_call() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let captured_variable = 5.0;

        let function = value::Function::new(&guard, Box::new(move |guard, info| {
            // Ensure the defaults are sensible
            assert!(info.is_construct_call == false);
            assert_eq!(info.arguments.len(), 2);
            assert_eq!(captured_variable, 5.0);

            let result = info.arguments[0].to_double(guard) +
                         info.arguments[1].to_double(guard) +
                         captured_variable;
            Ok(value::Number::from_double(guard, result).into())
        }));

        let result = function.call(&guard, &[
            &value::Number::new(&guard, 5).into(),
            &value::Number::from_double(&guard, 10.5).into()
        ]).unwrap();

        assert_eq!(result.to_integer(&guard), 20);
        assert_eq!(result.to_double(&guard), 20.5);
    }

    #[test]
    fn external_data_drop() {
        static mut called: bool = false;
        {
            struct Foo(i32);
            impl Drop for Foo {
                fn drop(&mut self) {
                    assert_eq!(self.0, 10);
                    unsafe { called = true };
                }
            }

            let (_runtime, context) = setup_env();
            let guard = context.make_current().unwrap();
            let _external = value::External::new(&guard, Box::new(Foo(10)));
        }
        assert!(unsafe { called });
    }

    #[test]
    fn error_object() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let error = value::Error::type_error(&guard, "FooBar");
        assert_eq!(error.to_string(&guard), "TypeError: FooBar");
    }

    #[test]
    fn array_iter() {
        let (_runtime, context) = setup_env();
        let guard = context.make_current().unwrap();
        let array = value::Array::new(&guard, 10);

        for i in 0..10 {
            array.set_index(&guard, i, &value::Number::new(&guard, i as i32));
        }

        assert_eq!(array.iter(&guard).fold(0, |acc, value| acc + value.to_integer(&guard)), 45);
    }
}
