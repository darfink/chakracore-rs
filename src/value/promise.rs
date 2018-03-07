use chakracore_sys::*;
use context::ContextGuard;
use super::{Value, Object, Function};
use error::*;
use {util, Context};

/// A JavaScript promise executor.
pub struct Executor {
    resolve: Function,
    reject: Function,
}

impl Executor {
    /// Consumes the `Executor` and fulfills the associated promise.
    pub fn resolve(self, guard: &ContextGuard, arguments: &[&Value]) -> Result<()> {
        self.resolve.call(guard, arguments).map(|_| ())
    }

    /// Consumes the `Executor` and rejects the associated promise.
    pub fn reject(self, guard: &ContextGuard, arguments: &[&Value]) -> Result<()> {
        self.reject.call(guard, arguments).map(|_| ())
    }
}

/// A JavaScript promise.
///
/// To support promises within a context, see [Context](../context/struct.Context.html).
pub struct Promise(JsValueRef);

impl Promise {
    /// Creates a new promise with an associated executor.
    pub fn new(_guard: &ContextGuard) -> (Self, Executor) {
        let mut reference = JsValueRef::new();
        let mut resolve = JsValueRef::new();
        let mut reject = JsValueRef::new();

        unsafe {
            jsassert!(JsCreatePromise(&mut reference, &mut resolve, &mut reject));
            (Self::from_raw(reference), Executor {
                resolve: Function::from_raw(resolve),
                reject: Function::from_raw(reject)
            })
        }
    }

    /// Returns true if the value is a `Promise`.
    pub fn is_same(value: &Value) -> bool {
        // See: https://github.com/Microsoft/ChakraCore/issues/135
        // There is no straight foward way to do this with the current API.
        value.clone().into_object().map_or(false, |object| {
            Context::exec_with_value(&object, |guard| {
                let promise = util::jsfunc(guard, "Promise")
                    .expect("retrieving Promise constructor");
                object.instance_of(guard, &promise)
            })
            .expect("changing active context for Promise comparison")
            .expect("missing associated context for Promise comparison")
        })
    }
}

reference!(Promise);
inherit!(Promise, Object);
subtype!(Promise, Value);

#[cfg(test)]
mod tests {
    use {test, value, script, Property};

    #[test]
    fn resolve() {
        test::run_with_context(|guard| {
            let (promise, executor) = value::Promise::new(guard);
            executor.resolve(guard, &[&value::Number::new(guard, 10)]).unwrap();

            let property = Property::new(guard, "promise");
            guard.global().set(guard, &property, &promise);

            let result = script::eval(guard, "
                var result = {};
                promise.then(function(value) { result.val = value; });
                result")
                .unwrap()
                .into_object()
                .unwrap();
            guard.execute_tasks();
            assert_eq!(result.get(guard, &Property::new(guard, "val")).to_integer(guard), 10);
        });
    }

    #[test]
    fn conversion() {
        test::run_with_context(|guard| {
            let (promise, _) = value::Promise::new(guard);
            let value: value::Value = promise.into();
            assert!(value.into_promise().is_some());

            let promise = script::eval(guard, "new Promise(() => {})")
                .unwrap()
                .into_promise();
            assert!(promise.is_some());
        });
    }
}
