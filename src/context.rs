//! Execution contexts and sandboxing.
use std::marker::PhantomData;
use std::ptr;
use error::*;
use jsrt_sys::*;
use value;
use Runtime;

/// A sandboxed execution context with its own set of built-in objects and functions.
#[derive(Clone, Debug)]
pub struct Context(JsContextRef);

// TODO: Use a stack for active contexts?
impl Context {
    /// Creates a new context and returns a handle to it.
    pub fn new(runtime: &Runtime) -> Result<Context> {
        let mut reference = JsContextRef::new();
        unsafe {
            jstry!(JsCreateContext(runtime.as_raw(), &mut reference));
            Ok(Context::from_raw(reference))
        }
    }

    /// Creates a context from a raw pointer.
    pub unsafe fn from_raw(reference: JsContextRef) -> Context {
        Context(reference)
    }

    /// Returns an object's associated context.
    pub unsafe fn from_object(object: &value::Object) -> Result<Context> {
        let mut reference = JsContextRef::new();
        jstry!(JsGetContextOfObject(object.as_raw(), &mut reference));
        Ok(Context::from_raw(reference))
    }

    /// Binds the context to the current scope.
    ///
    /// The majority of APIs require an active context.
    pub fn make_current<'a>(&'a self) -> Result<ContextGuard<'a>> {
        self.enter().map(|_| {
            ContextGuard::<'a> {
                context: self.clone(),
                phantom: PhantomData,
                drop: true,
            }
        })
    }

    /// Returns the active context in the current thread.
    ///
    /// This is unsafe because there should be no reason to use it in idiomatic
    /// code. Usage patterns should utilize `ContextGuard` instead. The
    /// associated lifetime has no connection to an actual `Context`.
    ///
    /// This `ContextGuard` does not reset the current context upon destruction,
    /// in contrast to a normally allocated `ContextGuard`, this is merely a
    /// reference.
    pub unsafe fn get_current<'a>() -> Result<ContextGuard<'a>> {
        let mut reference = JsContextRef::new();
        jstry!(JsGetCurrentContext(&mut reference));
        Ok(ContextGuard {
            context: Context::from_raw(reference),
            phantom: PhantomData,
            drop: false,
        })
    }

    /// Sets the internal data of the context.
    pub fn set_data<'a, T>(&'a self, data: &'a mut T) -> Result<()> {
        jstry!(unsafe { JsSetContextData(self.as_raw(), data as *mut _ as *mut _) });
        Ok(())
    }

    /// Gets the internal data of the context.
    pub unsafe fn get_data<T>(&self) -> Result<*mut T> {
        let mut data = ptr::null_mut();
        jstry!(JsGetContextData(self.as_raw(), &mut data));
        Ok(data as *mut T)
    }

    /// Returns the underlying raw pointer.
    pub fn as_raw(&self) -> JsContextRef {
        self.0
    }

    /// Sets the current context.
    fn enter(&self) -> Result<()> {
        jstry!(unsafe { JsSetCurrentContext(self.0) });
        Ok(())
    }

    /// Unsets the current context.
    fn exit(&self) -> Result<()> {
        // This is called from a destructor, so assert instead of returning the error
        jstry!(unsafe { JsSetCurrentContext(JsValueRef::new()) });
        Ok(())
    }
}

/// A guard that keeps a context active while it is in scope.
#[must_use]
pub struct ContextGuard<'a> {
    context: Context,
    phantom: PhantomData<&'a Context>,
    drop: bool,
}

impl<'a> ContextGuard<'a> {
    /// Returns the guard's associated context.
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// Returns the active context's global object.
    pub fn global(&self) -> Result<value::Object> {
        let mut value = JsValueRef::new();
        unsafe {
            jstry!({
                JsGetGlobalObject(&mut value)
            });
            Ok(value::Object::from_raw(value))
        }
    }
}

impl<'a> Drop for ContextGuard<'a> {
    /// Resets the currently active context.
    fn drop(&mut self) {
        if self.drop {
            assert!(self.context.exit().is_ok())
        }
    }
}
