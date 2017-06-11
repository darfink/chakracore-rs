//! A JavaScript function and associated types.
use std::{slice, mem};
use libc::{c_void, c_ushort};
use chakracore_sys::*;
use context::{Context, ContextGuard};
use error::*;
use util;
use super::{Value, Object};

/// The information passed to `FunctionCallback` closures.
#[derive(Clone, Debug)]
pub struct CallbackInfo {
    /// Whether it's a constructor call or not.
    pub is_construct_call: bool,
    /// Arguments supplied by the caller.
    pub arguments: Vec<Value>,
    /// The source of this function call.
    pub callee: Value,
    /// The function's `this` context.
    pub this: Value,
}

/// The result returned from a function callback.
pub type CallbackResult = ::std::result::Result<Value, Value>;

/// Callback type for functions.
pub type FunctionCallback =
    Fn(&ContextGuard, CallbackInfo) -> CallbackResult + 'static;

/// A JavaScript function object.
pub struct Function(JsValueRef);

impl Function {
    /// Creates an anonymous function
    pub fn new(_guard: &ContextGuard, callback: Box<FunctionCallback>) -> Self {
        Self::create(callback, |context, reference| unsafe {
            JsCreateFunction(Some(Self::callback), context, reference)
        })
    }

    /// Creates a named function
    pub fn with_name(guard: &ContextGuard, name: &str, callback: Box<FunctionCallback>) -> Self {
        Self::create(callback, |context, reference| unsafe {
            let name = super::String::new(guard, name);
            JsCreateNamedFunction(name.as_raw(), Some(Self::callback), context, reference)
        })
    }

    /// Returns whether the object is an instance of this `Function` or not.
    pub fn instance_of(&self, _guard: &ContextGuard, object: super::Object) -> Result<bool> {
        let mut result = false;
        unsafe {
            jstry!(JsInstanceOf(object.as_raw(), self.as_raw(), &mut result));
            Ok(result)
        }
    }

    /// Calls a function and returns the result. The context (i.e `this`) will
    /// be the global object associated with the `ContextGuard`.
    pub fn call(&self, guard: &ContextGuard, arguments: &[&Value]) -> Result<Value> {
        self.call_with_this(guard, &guard.global().into(), arguments)
    }

    /// Calls a function, with a context, and returns the result.
    pub fn call_with_this(&self, _guard: &ContextGuard, this: &Value, arguments: &[&Value]) -> Result<Value> {
        self.invoke(_guard, this, arguments, false)
    }

    /// Calls a function as a constructor and returns the result.
    pub fn construct(&self, _guard: &ContextGuard, this: &Value, args: &[&Value]) -> Result<Value> {
        self.invoke(_guard, this, args, true)
    }

    is_same!(Function, "Returns true if the value is a `Function`.");

    /// Invokes a function and returns the result.
    fn invoke(&self,
              guard: &ContextGuard,
              this: &Value,
              arguments: &[&Value],
              constructor: bool)
              -> Result<Value> {
        // Combine the context with the arguments
        let mut forward = Vec::with_capacity(arguments.len() + 1);
        forward.push(this.as_raw());
        forward.extend(arguments.iter().map(|value| value.as_raw()));

        let api = if constructor {
            JsConstructObject
        } else {
            JsCallFunction
        };

        unsafe {
            let mut result = JsValueRef::new();
            let code = api(self.0,
                           forward.as_mut_ptr(),
                           forward.len() as c_ushort,
                           &mut result);
            util::handle_exception(guard, code).map(|_| Value::from_raw(result))
        }
    }

    /// Prevents boilerplate code in constructors.
    fn create<T>(callback: Box<FunctionCallback>, initialize: T) -> Self
        where T: FnOnce(*mut c_void, &mut JsValueRef) -> JsErrorCode
    {
        // Because a boxed callback can be a fat pointer, it needs to be wrapped
        // in an additional Box to ensure it fits in a single pointer.
        let wrapper = Box::into_raw(Box::new(callback));

        unsafe {
            let mut reference = JsValueRef::new();
            jsassert!(initialize(wrapper as *mut _, &mut reference));
            let function = Self::from_raw(reference);

            // Ensure the heap objects are freed
            function.set_collect_callback(Box::new(move |_| {
                Box::from_raw(wrapper);
            }));
            function
        }
    }

    /// A function callback, triggered on call.
    unsafe extern "system" fn callback(callee: JsValueRef,
                                       is_construct_call: bool,
                                       arguments: *mut JsValueRef,
                                       len: c_ushort,
                                       data: *mut c_void)
                                       -> *mut c_void {
        // This memory is cleaned up during object collection
        let callback = data as *mut Box<FunctionCallback>;

        // There is always an active context in callbacks
        let guard = Context::get_current().unwrap();

        // Construct the callback information object
        let arguments = slice::from_raw_parts_mut(arguments, len as usize);
        let info = CallbackInfo {
            is_construct_call: is_construct_call,
            arguments: arguments[1..].iter().map(|value| Value::from_raw(*value)).collect(),
            callee: Value::from_raw(callee),
            this: Value::from_raw(arguments[0]),
        };

        // Call the user supplied callback
        match (*callback)(&guard, info) {
            Ok(value) => mem::transmute(value.as_raw()),
            Err(error) => {
                jsassert!(JsSetException(error.as_raw()));
                mem::transmute(error.as_raw())
            }
        }
    }
}

reference!(Function);
inherit!(Function, Object);
subtype!(Function, Value);

#[cfg(test)]
mod tests {
    use {test, value, script, Property};

    #[test]
    fn multiply() {
        test::run_with_context(|guard| {
            let captured_variable = 5.0;
            let function = value::Function::new(guard, Box::new(move |guard, info| {
                // Ensure the defaults are sensible
                assert_eq!(info.is_construct_call, false);
                assert_eq!(info.arguments.len(), 2);
                assert_eq!(captured_variable, 5.0);

                let result = info.arguments[0].to_double(guard)
                           + info.arguments[1].to_double(guard)
                           + captured_variable;
                Ok(value::Number::from_double(guard, result).into())
            }));

            let result = function.call(guard, &[
                &value::Number::new(guard, 5).into(),
                &value::Number::from_double(guard, 10.5).into()
            ]).unwrap();

            assert_eq!(result.to_integer(guard), 20);
            assert_eq!(result.to_double(guard), 20.5);
        });
    }

    #[test]
    fn exception() {
        test::run_with_context(|guard| {
            let function = value::Function::new(guard, Box::new(move |guard, _| {
                Err(value::Error::new(guard, "Exception").into())
            }));

            let global = guard.global();
            let property = Property::new(guard, "test");
            global.set(guard, &property, &function);

            let result = script::eval(guard,
                "try { test(); } catch (ex) { ex.message; }").unwrap();

            assert_eq!(result.to_string(guard), "Exception");
        });
    }
}