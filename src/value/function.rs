use std::slice;
use libc::{c_void, c_ushort};
use chakra_sys::*;
use context::{Context, ContextGuard};
use error::*;
use super::Value;

struct FunctionThunk {
    callback: *mut FunctionCallback,
}

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
pub type FunctionCallback = Fn(&ContextGuard, &CallbackInfo) -> ::std::result::Result<Value, Value> + 'static;

/// A JavaScript function object.
#[derive(Clone, Debug)]
pub struct Function(JsValueRef);

impl Function {
    /// Creates an anonymous function
    pub fn new(_guard: &ContextGuard, callback: Box<FunctionCallback>) -> Result<Self> {
        let mut reference = JsValueRef::new();
        let thunk = Box::new(FunctionThunk { callback: Box::into_raw(callback) });

        unsafe {
            // TODO: Handle optional names (`JsCreateNamedFunction`, fn with_name)
            jstry!(JsCreateFunction(Some(Self::callback),
                                    Box::into_raw(thunk) as *mut _,
                                    &mut reference));
            Ok(Function(reference))
        }
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
    pub fn construct(&self, _guard: &ContextGuard, this: &Value, arguments: &[Value]) -> Result<Value> {
        self.invoke(_guard, this, arguments, true)
    }

    /// Invokes a function and returns the result.
    fn invoke(&self,
              _guard: &ContextGuard,
              this: &Value,
              arguments: &[Value],
              is_construct_call: bool) -> Result<Value> {
        // Combine the context with the arguments
        let mut forward = vec![this.as_raw()];
        forward.extend(arguments.iter().map(|value| value.as_raw()));

        let api = if is_construct_call {
            JsConstructObject
        } else {
            JsCallFunction
        };

        unsafe {
            // TODO: handle exceptions!
            let mut result = JsValueRef::new();
            jstry!(api(self.0, forward.as_mut_ptr(), forward.len() as c_ushort, &mut result));
            Ok(Value::from_raw(result))
        }
    }

    /// Function implementation for callbacks
    extern "system" fn callback(callee: JsValueRef,
                                is_construct_call: bool,
                                arguments: *mut JsValueRef,
                                len: c_ushort,
                                state: *mut c_void) -> JsValueRef {
        // TODO: When and how should the heap memory be freed.
        let thunk = unsafe { (state as *mut FunctionThunk).as_ref().unwrap() };

        unsafe {
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
            match (*thunk.callback)(&guard, &info) {
                Ok(value) => value.as_raw(),
                Err(error) => {
                    // TODO: what is best to return here? Undefined or exception.
                    assert_eq!(JsSetException(error.as_raw()), JsErrorCode::NoError);
                    error.as_raw()
                },
            }
        }
    }
}

inherit!(Function, Value);
