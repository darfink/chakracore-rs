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
    promise_queue: Vec<value::Function>,
    user_data: AnyMap,
}

/// A sandboxed execution context with its own set of built-in objects and functions.
#[derive(Debug)]
pub struct Context(JsContextRef);

// TODO: Should context lifetime explicitly depend on runtime?
impl Context {
    /// Creates a new context and returns a handle to it.
    pub fn new(runtime: &Runtime) -> Result<Context> {
        let mut reference = JsContextRef::new();
        unsafe {
            jstry!(JsCreateContext(runtime.as_raw(), &mut reference));
            jstry!(JsSetObjectBeforeCollectCallback(reference, ptr::null_mut(), Some(Self::collect)));

            let context = Self::from_raw(reference);
            context.set_data(Box::new(ContextData {
                promise_queue: Vec::new(),
                user_data: AnyMap::new(),
            }))?;

            /* Promise continuation callback requires an active context */
            {
                let _guard = context.make_current();
                jstry!(JsSetPromiseContinuationCallback(
                    Some(Self::promise_handler), context.get_data() as *mut _ as *mut _));
            }

            Ok(context)
        }
    }

    /// Returns an object's associated context.
    pub unsafe fn from_object(object: &value::Object) -> Result<Context> {
        let mut reference = JsContextRef::new();
        jstry!(JsGetContextOfObject(object.as_raw(), &mut reference));
        Ok(Self::from_raw(reference))
    }

    /// Binds the context to the current scope.
    ///
    /// The majority of APIs require an active context.
    pub fn make_current<'a>(&'a self) -> Result<ContextGuard<'a>> {
        // Preserve the previous context so it can be restored later
        let current = unsafe { Self::get_current().map(|guard| guard.current.clone()) };

        self.enter().map(|_| {
            ContextGuard::<'a> {
                previous: current,
                current: self.clone(),
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
    pub unsafe fn get_current<'a>() -> Option<ContextGuard<'a>> {
        let mut reference = JsContextRef::new();
        jsassert!(JsGetCurrentContext(&mut reference));

        // The JSRT API returns null instead of an error code
        reference.0.as_ref().map(|_| ContextGuard {
            previous: None,
            current: Self::from_raw(reference),
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
    fn exit(&self, previous: Option<&Context>) -> Result<()> {
        jstry!(unsafe {
            let next = previous
                .map(|context| context.as_raw())
                .unwrap_or(JsValueRef::new());
            JsSetCurrentContext(next)
        });
        Ok(())
    }

    /// A promise handler, triggered whenever a promise method is used.
    unsafe extern "system" fn promise_handler(task: JsValueRef, data: *mut ::libc::c_void) {
        let data = (data as *mut ContextData).as_mut().unwrap();
        data.promise_queue.push(value::Function::from_raw(task));
    }

    /// A collect callback, triggered before the context is destroyed.
    unsafe extern "system" fn collect(context: JsContextRef, _: *mut ::libc::c_void) {
        let context = Self::from_raw(context);
        Box::from_raw(context.get_data());
    }
}

reference!(Context);

/// A guard that keeps a context active while it is in scope.
#[must_use]
#[derive(Debug)]
pub struct ContextGuard<'a> {
    previous: Option<Context>,
    current: Context,
    phantom: PhantomData<&'a Context>,
    drop: bool,
}

impl<'a> ContextGuard<'a> {
    /// Returns the guard's associated context.
    pub fn context(&self) -> Context {
        self.current.clone()
    }

    /// Returns the active context's global object.
    pub fn global(&self) -> value::Object {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsGetGlobalObject(&mut value));
            value::Object::from_raw(value)
        }
    }

    /// Executes all the context's queued promise tasks.
    pub fn execute_tasks(&self) {
        let data = unsafe { self.current.get_data() };
        while let Some(task) = data.promise_queue.pop() {
            task.call(self, &[]).unwrap();
        }
    }
}

impl<'a> Drop for ContextGuard<'a> {
    /// Resets the currently active context.
    fn drop(&mut self) {
        if self.drop {
            assert!(self.current.exit(self.previous.as_ref()).is_ok())
        }
    }
}

#[cfg(test)]
mod tests {
    use {test, value, script, Context, Property};

    #[test]
    fn global() {
        test::run_with_context(|guard| {
            let global = guard.global();
            let dirname = Property::new(guard, "__dirname");

            global.set(guard, &dirname, &value::String::new(guard, "FooBar"));
            global.set_index(guard, 2, &value::Number::new(guard, 1337));

            let result1 = script::eval(guard, "__dirname").unwrap();
            let result2 = script::eval(guard, "this[2]").unwrap();

            assert_eq!(result1.to_string(guard), "FooBar");
            assert_eq!(result2.to_integer(guard), 1337);
       });
    }

    #[test]
    fn stack() {
        let (runtime, context) = test::setup_env();
        {
            let get_current_raw = || unsafe { Context::get_current().unwrap().context().as_raw() };
            let _guard = context.make_current().unwrap();

            assert_eq!(get_current_raw(), context.as_raw());
            {
                let inner_context = Context::new(&runtime).unwrap();
                let _guard = inner_context.make_current().unwrap();
                assert_eq!(get_current_raw(), inner_context.as_raw());
            }
            assert_eq!(get_current_raw(), context.as_raw());
        }
        assert!(unsafe { Context::get_current() }.is_none());
    }

    #[test]
    fn promise_queue() {
        test::run_with_context(|guard| {
            let result = script::eval(guard, "
                var object = {};
                Promise.resolve(5)
                    .then(val => val + 5)
                    .then(val => val / 5)
                    .then(val => object.val = val);
                object;").unwrap();

            guard.execute_tasks();

            let value = result
                .into_object()
                .unwrap()
                .get(guard, &Property::new(guard, "val"))
                .to_integer(guard);
            assert_eq!(value, 2);
        });
    }
}