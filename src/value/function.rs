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

/// Callback type for functions.
pub type FunctionCallback =
    Fn(&ContextGuard, &mut CallbackInfo) -> ::std::result::Result<Value, Value> + 'static;

/// A JavaScript function object.
#[derive(Clone, Debug)]
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
            let name = super::String::from_str(guard, name);
            JsCreateNamedFunction(name.as_raw(), Some(Self::callback), context, reference)
        })
    }

    /// Creates a function from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Function(reference)
    }

    /// Returns whether the `object` is an instance of this `Function`.
    pub fn instance_of(&self, _guard: &ContextGuard, object: super::Object) -> Result<bool> {
        let mut result = false;
        unsafe {
            jstry!(JsInstanceOf(object.as_raw(), self.as_raw(), &mut result));
            Ok(result)
        }
    }

    /// Calls a function and returns the result.
    pub fn call(&self, _guard: &ContextGuard, this: &Value, arguments: &[Value]) -> Result<Value> {
        self.invoke(_guard, this, arguments, false)
    }

    /// Calls a function as a constructor and returns the result.
    pub fn construct(&self, _guard: &ContextGuard, this: &Value, args: &[Value]) -> Result<Value> {
        self.invoke(_guard, this, args, true)
    }

    /// Invokes a function and returns the result.
    fn invoke(&self,
              guard: &ContextGuard,
              this: &Value,
              arguments: &[Value],
              constructor: bool)
              -> Result<Value> {
        // Combine the context with the arguments
        let mut forward = vec![this.as_raw()];
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
            let function = Function::from_raw(reference);

            // Ensure the heap objects are freed
            function.set_collect_callback(Box::new(move |_| {
                Box::from_raw(wrapper);
            }));
            function
        }
    }

    /// Function implementation for callbacks
    unsafe extern "system" fn callback(callee: JsValueRef,
                                       is_construct_call: bool,
                                       arguments: *mut JsValueRef,
                                       len: c_ushort,
                                       data: *mut c_void)
                                       -> *mut c_void {
        // This memory is cleaned up during object collection
        let callback = data as *mut Box<FunctionCallback>;

        // There is always an active context in this case
        let guard = Context::get_current().unwrap();

        // Construct the callback information object
        let arguments = slice::from_raw_parts_mut(arguments, len as usize);
        let mut info = CallbackInfo {
            is_construct_call: is_construct_call,
            arguments: arguments[1..].iter().map(|value| Value::from_raw(*value)).collect(),
            callee: Value::from_raw(callee),
            this: Value::from_raw(arguments[0]),
        };

        // Call the user supplied callback
        match (*callback)(&guard, &mut info) {
            Ok(value) => mem::transmute(value.as_raw()),
            Err(error) => {
                // TODO: what is best to return here? Undefined or exception.
                jsassert!(JsSetException(error.as_raw()));
                mem::transmute(error.as_raw())
            }
        }
    }
}

inherit!(Function, Object);
subtype!(Function, Value);
