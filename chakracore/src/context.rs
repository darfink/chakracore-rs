//! Execution contexts and sandboxing.
use anymap::AnyMap;
use boolinator::Boolinator;
use chakracore_sys::*;
use error::*;
use std::marker::PhantomData;
use std::ptr;
use util::jstry;
use {value, Runtime};

/// Used for holding context instance data.
struct ContextData {
  promise_queue: Vec<value::Function>,
  user_data: AnyMap,
}

/// A sandboxed execution context with its own set of built-in objects and
/// functions.
///
/// The majority of APIs require an active context.
///
/// In a browser or Node.JS environment, the task of executing promises is
/// handled by the runtime. This is not the case with **ChakraCore**. To run
/// promise chains, `execute_tasks` must be called at a regular interval. This
/// is done using the `ContextGuard`.
#[derive(Debug, PartialEq)]
pub struct Context(JsContextRef);

// TODO: Should context lifetime explicitly depend on runtime?
impl Context {
  /// Creates a new context and returns a handle to it.
  pub fn new(runtime: &Runtime) -> Result<Context> {
    let mut reference = JsContextRef::new();
    unsafe {
      jstry!(JsCreateContext(runtime.as_raw(), &mut reference));
      jstry!(JsSetObjectBeforeCollectCallback(
        reference,
        ptr::null_mut(),
        Some(Self::collect)
      ));

      let context = Self::from_raw(reference);
      context.set_data(Box::new(ContextData {
        promise_queue: Vec::new(),
        user_data: AnyMap::new(),
      }))?;

      // Promise continuation callback requires an active context
      context
        .exec_with(|_| {
          let data = context.get_data() as *mut _ as *mut _;
          jstry(JsSetPromiseContinuationCallback(
            Some(Self::promise_handler),
            data,
          ))
        })
        .expect("activating promise continuation callback")
        .map(|_| context)
    }
  }

  /// Binds the context to the current scope.
  pub fn make_current<'a>(&'a self) -> Result<ContextGuard<'a>> {
    // Preserve the previous context so it can be restored later
    let current = unsafe { Self::get_current().map(|guard| guard.current.clone()) };

    self.enter().map(|_| ContextGuard::<'a> {
      previous: current,
      current: self.clone(),
      phantom: PhantomData,
      drop: true,
    })
  }

  /// Returns the active context in the current thread.
  ///
  /// This is unsafe because there should be little reason to use it in
  /// idiomatic code.
  ///
  /// Usage patterns should utilize `ContextGuard` or
  /// `exec_with_current` instead.
  ///
  /// This `ContextGuard` does not reset the current context upon destruction,
  /// in contrast to a normally allocated `ContextGuard`. This is merely a
  /// hollow reference.
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

  /// Binds the context to the closure's scope.
  ///
  /// ```c
  /// let result = context.exec_with(|guard| script::eval(guard, "1 + 1")).unwrap();
  /// ```
  pub fn exec_with<Ret, T: FnOnce(&ContextGuard) -> Ret>(&self, callback: T) -> Result<Ret> {
    self.make_current().map(|guard| callback(&guard))
  }

  /// Executes a closure with the thread's active context.
  ///
  /// This is a safe alternative to `get_current`. It will either return the
  /// closures result wrapped in `Some`, or `None`, if no context is currently
  /// active.
  pub fn exec_with_current<Ret, T: FnOnce(&ContextGuard) -> Ret>(callback: T) -> Option<Ret> {
    unsafe { Self::get_current().as_ref().map(callback) }
  }

  /// Executes a closure with a value's associated context.
  ///
  /// - The active context will only be changed if it differs from the value's.
  /// - If the switch fails, an error will be returned.
  /// - Due to the fact that this relies on `from_value`, it suffers from the
  ///   same limitations and should be avoided.
  /// - If the value has no associated context, `None` will be returned.
  pub(crate) fn exec_with_value<T, Ret>(value: &value::Value, callback: T) -> Result<Option<Ret>>
  where
    T: FnOnce(&ContextGuard) -> Ret,
  {
    Context::from_value(value).map_or(Ok(None), |context| unsafe {
      // In case there is no active context, or if it differs from the
      // value's context, temporarily change the context.
      let guard = Context::get_current()
        .and_then(|guard| (guard.context() == context).as_some(guard))
        .map_or_else(|| context.make_current(), Ok);
      guard.map(|guard| Some(callback(&guard)))
    })
  }

  /// Set user data associated with the context.
  ///
  /// - Only one value per type.
  /// - The internal implementation uses `AnyMap`.
  /// - Returns a previous value if applicable.
  /// - The data will live as long as the runtime keeps the context.
  pub fn insert_user_data<T>(&self, value: T) -> Option<T>
  where
    T: Send + 'static,
  {
    unsafe { self.get_data().user_data.insert(value) }
  }

  /// Remove user data associated with the context.
  pub fn remove_user_data<T>(&self) -> Option<T>
  where
    T: Send + 'static,
  {
    unsafe { self.get_data().user_data.remove::<T>() }
  }

  /// Get user data associated with the context.
  pub fn get_user_data<T>(&self) -> Option<&T>
  where
    T: Send + 'static,
  {
    unsafe { self.get_data().user_data.get::<T>() }
  }

  /// Get mutable user data associated with the context.
  pub fn get_user_data_mut<T>(&self) -> Option<&mut T>
  where
    T: Send + 'static,
  {
    unsafe { self.get_data().user_data.get_mut::<T>() }
  }

  /// Returns a recyclable value's associated context.
  ///
  /// This is unreliable, because types that have an associated context is
  /// implementation defined (by the underlying runtime), based on whether they
  /// are recyclable or not, therefore it should be avoided.
  fn from_value(value: &value::Value) -> Option<Context> {
    let mut reference = JsContextRef::new();
    unsafe {
      jstry(JsGetContextOfObject(value.as_raw(), &mut reference))
        .ok()
        .map(|_| Self::from_raw(reference))
    }
  }

  /// Sets the internal data of the context.
  unsafe fn set_data(&self, data: Box<ContextData>) -> Result<()> {
    jstry(JsSetContextData(
      self.as_raw(),
      Box::into_raw(data) as *mut _,
    ))
  }

  /// Gets the internal data of the context.
  unsafe fn get_data<'a>(&'a self) -> &'a mut ContextData {
    let mut data = ptr::null_mut();
    jsassert!(JsGetContextData(self.as_raw(), &mut data));
    (data as *mut ContextData)
      .as_mut()
      .expect("retrieving context data")
  }

  /// Sets the current context.
  fn enter(&self) -> Result<()> {
    jstry(unsafe { JsSetCurrentContext(self.as_raw()) })
  }

  /// Unsets the current context.
  fn exit(&self, previous: Option<&Context>) -> Result<()> {
    jstry(unsafe {
      let next = previous
        .map(|context| context.as_raw())
        .unwrap_or_else(JsValueRef::new);
      JsSetCurrentContext(next)
    })
  }

  /// A promise handler, triggered whenever a promise method is used.
  unsafe extern "system" fn promise_handler(task: JsValueRef, data: *mut ::libc::c_void) {
    let data = (data as *mut ContextData)
      .as_mut()
      .expect("retrieving promise handler stack");
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
      task.call(self, &[]).expect("executing promise task");
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
  use {script, test, value, Context, Property};

  #[test]
  fn global() {
    test::run_with_context(|guard| {
      let global = guard.global();
      let dirname = Property::new(guard, "__dirname");

      global.set(guard, &dirname, value::String::new(guard, "FooBar"));
      global.set_index(guard, 2, value::Number::new(guard, 1337));

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
      let get_current = || unsafe { Context::get_current().unwrap().context() };
      let _guard = context.make_current().unwrap();

      assert_eq!(get_current(), context);
      {
        let inner_context = Context::new(&runtime).unwrap();
        let _guard = inner_context.make_current().unwrap();
        assert_eq!(get_current(), inner_context);
      }
      assert_eq!(get_current(), context);
    }
    assert!(unsafe { Context::get_current() }.is_none());
  }

  #[test]
  fn user_data() {
    test::run_with_context(|guard| {
      type Data = Vec<i32>;
      let context = guard.context();

      let data: Data = vec![10, 20];
      context.insert_user_data(data);

      let data = context.get_user_data::<Data>().unwrap();
      assert_eq!(data.as_slice(), [10, 20]);

      assert!(context.remove_user_data::<Data>().is_some());
      assert!(context.get_user_data::<Data>().is_none());
    });
  }

  #[test]
  fn promise_queue() {
    test::run_with_context(|guard| {
      let result = script::eval(
        guard,
        "
                var object = {};
                Promise.resolve(5)
                    .then(val => val + 5)
                    .then(val => val / 5)
                    .then(val => object.val = val);
                object;",
      )
      .unwrap();

      guard.execute_tasks();

      let value = result
        .into_object()
        .unwrap()
        .get(guard, Property::new(guard, "val"))
        .to_integer(guard);
      assert_eq!(value, 2);
    });
  }

  #[test]
  fn shared_objects() {
    let (runtime, context) = test::setup_env();
    let context2 = Context::new(&runtime).unwrap();

    let guard1 = context.make_current().unwrap();
    let object = script::eval(&guard1, "({ foo: 1337 })").unwrap();

    let guard2 = context2.make_current().unwrap();
    assert_eq!(object.to_json(&guard2).unwrap(), r#"{"foo":1337}"#);
  }
}
