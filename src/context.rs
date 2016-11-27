//! Execution contexts and sandboxing.
use std::marker::PhantomData;
use std::ptr;
use anymap::AnyMap;
use error::*;
use chakracore_sys::*;
use value;
use Runtime;

/// Used for holding context instance data.
struct ContextData {
    user_data: AnyMap,
}

/// A sandboxed execution context with its own set of built-in objects and functions.
#[derive(Clone, Debug)]
pub struct Context(JsContextRef);

// TODO: Use a stack for active contexts?
// TODO: Should context lifetime explicitly depend on runtime?
impl Context {
    /// Creates a new context and returns a handle to it.
    pub fn new(runtime: &Runtime) -> Result<Context> {
        let mut reference = JsContextRef::new();
        unsafe {
            jstry!(JsCreateContext(runtime.as_raw(), &mut reference));
            jstry!(JsSetObjectBeforeCollectCallback(reference, ptr::null_mut(), Some(Self::collect)));

            let context = Context::from_raw(reference);
            context.set_data(Box::new(ContextData {
                user_data: AnyMap::new()
            }))?;

            Ok(context)
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

    /// Set user data associated with the context. Only one value per type. The
    /// internal implementation uses `AnyMap`. Returns a previous value if
    /// applicable. The data will live as long as the runtime keeps the context.
    pub fn insert_user_data<T>(&self, value: T) -> Option<T> where T: 'static {
        unsafe { self.get_data().user_data.insert(value) }
    }

    /// Remove user data associated with the context.
    pub fn remove_user_data<T>(&self) -> Option<T> where T: 'static {
        unsafe { self.get_data().user_data.remove::<T>() }
    }

    /// Get user data associated with the context.
    pub fn get_user_data<T>(&self) -> Option<&T> where T: 'static {
        unsafe { self.get_data().user_data.get::<T>() }
    }

    /// Get mutable user data associated with the context.
    pub fn get_user_data_mut<T>(&self) -> Option<&mut T> where T: 'static {
        unsafe { self.get_data().user_data.get_mut::<T>() }
    }

    /// Returns the active context in the current thread.
    ///
    /// This is unsafe because there should be no reason to use it in idiomatic
    /// code. Usage patterns should utilize `ContextGuard` instead. The
    /// associated lifetime has no connection to an actual `Context`.
    ///
    /// This `ContextGuard` does not reset the current context upon destruction,
    /// in contrast to a normally allocated `ContextGuard`. This is merely a
    /// reference.
    pub unsafe fn get_current<'a>() -> Result<ContextGuard<'a>> {
        let mut reference = JsContextRef::new();
        jstry!(JsGetCurrentContext(&mut reference));

        if reference.0.is_null() {
            // For some reason this check is not done by default
            jsassert!(JsErrorCode::NoCurrentContext, "JsGetCurrentContext");
        }

        Ok(ContextGuard {
            context: Context::from_raw(reference),
            phantom: PhantomData,
            drop: false,
        })
    }

    /// Sets the internal data of the context.
    unsafe fn set_data(&self, data: Box<ContextData>) -> Result<()> {
        jstry!(JsSetContextData(self.as_raw(), Box::into_raw(data) as *mut _));
        Ok(())
    }

    /// Gets the internal data of the context.
    unsafe fn get_data<'a>(&'a self) -> &'a mut ContextData {
        let mut data = ptr::null_mut();
        jsassert!(JsGetContextData(self.as_raw(), &mut data));
        (data as *mut _).as_mut().unwrap()
    }

    /// Returns the underlying raw pointer.
    pub fn as_raw(&self) -> JsContextRef {
        self.0
    }

    /// Sets the current context.
    fn enter(&self) -> Result<()> {
        jstry!(unsafe { JsSetCurrentContext(self.as_raw()) });
        Ok(())
    }

    /// Unsets the current context.
    fn exit(&self) -> Result<()> {
        // This is called from a destructor, so assert instead of returning the error
        jstry!(unsafe { JsSetCurrentContext(JsValueRef::new()) });
        Ok(())
    }

    /// A collect callback, triggered before the context is destroyed.
    unsafe extern "system" fn collect(context: JsContextRef, _: *mut ::libc::c_void) {
        let context = Context::from_raw(context);
        Box::from_raw(context.get_data());
    }
}

/// A guard that keeps a context active while it is in scope.
#[must_use]
#[derive(Debug)]
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
    pub fn global(&self) -> value::Object {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsGetGlobalObject(&mut value));
            value::Object::from_raw(value)
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
