//! Runtime and builder.
use crate::{util::jstry, Result};
use chakracore_sys::*;
use libc::c_void;
use std::time::{Duration, Instant};

/// A callback triggered before objects are collected.
pub type CollectCallback = dyn Fn() + Send;

/// A builder for the runtime type.
pub struct Builder {
  memory_limit: Option<usize>,
  collect_callback: Option<Box<CollectCallback>>,
  attributes: JsRuntimeAttributes,
}

/// An isolated instance of a runtime.
pub struct Runtime {
  #[allow(dead_code)]
  callback: Option<Box<Box<CollectCallback>>>,
  handle: JsRuntimeHandle,
  last_idle_tick: Option<Duration>,
  last_idle: Option<Instant>,
}

impl Runtime {
  /// Creates a new runtime.
  pub fn new() -> Result<Runtime> {
    Self::builder().build()
  }

  /// Returns a runtime builder.
  pub fn builder() -> Builder {
    Builder {
      memory_limit: None,
      collect_callback: None,
      attributes: JsRuntimeAttributeNone,
    }
  }

  /// Performs a full garbage collection.
  pub fn collect(&self) -> Result<()> {
    jstry(unsafe { JsCollectGarbage(self.as_raw()) })
  }

  /// Runs any idle tasks that are in the queue. The returned duration is the
  /// least amount of time that should pass until this function is called
  /// again. This call will fail if the runtime was created without idle
  /// processing enabled.
  ///
  /// Returns whether any processing was done or not.
  pub fn run_idle_tasks(&mut self) -> Result<bool> {
    let should_idle = self.last_idle.map_or(true, |before| {
      // Assume that `last_idle_tick` is set, if `last_idle` is
      Instant::now().duration_since(before) >= self.last_idle_tick.unwrap()
    });

    if should_idle {
      let mut ticks = 0;
      jstry(unsafe { JsIdle(&mut ticks) })?;

      self.last_idle_tick = Some(Duration::from_millis(ticks as u64));
      self.last_idle = Some(Instant::now());
    }

    Ok(should_idle)
  }

  /// Returns the runtime's memory usage
  pub fn get_memory_usage(&self) -> usize {
    let mut usage = 0;
    jsassert!(unsafe { JsGetRuntimeMemoryUsage(self.as_raw(), &mut usage) });
    usage
  }

  /// Returns the underlying raw pointer behind this runtime.
  pub fn as_raw(&self) -> JsRuntimeHandle {
    self.handle
  }

  /// A collector callback, triggered before objects are released.
  unsafe extern "system" fn before_collect(data: *mut c_void) {
    let callback = data as *mut Box<CollectCallback>;
    (*callback)();
  }
}

impl Drop for Runtime {
  fn drop(&mut self) {
    unsafe {
      jsassert!(JsDisposeRuntime(self.as_raw()));
    }
  }
}

macro_rules! attr {
  ($name:ident, $attribute:ident, $doc:expr) => {
    #[doc=$doc]
    pub fn $name(mut self) -> Self {
      self.attributes = self.attributes | $attribute;
      self
    }
  };
}

impl Builder {
  attr!(
    disable_background_work,
    JsRuntimeAttributeDisableBackgroundWork,
    "Disable the runtime from doing any work on background threads."
  );
  attr!(
    disable_eval,
    JsRuntimeAttributeDisableEval,
    "Disable `eval` and `function` by throwing an exception upon use."
  );
  attr!(
    disable_jit,
    JsRuntimeAttributeDisableNativeCodeGeneration,
    "Disable just-in-time compilation."
  );
  attr!(
    enable_experimental,
    JsRuntimeAttributeEnableExperimentalFeatures,
    "Allow experimental JavaScript features."
  );
  attr!(
    enable_script_interrupt,
    JsRuntimeAttributeAllowScriptInterrupt,
    "Allow script interrupt."
  );
  attr!(
    dispatch_exceptions,
    JsRuntimeAttributeDispatchSetExceptionsToDebugger,
    "Dispatch exceptions to any attached JavaScript debuggers."
  );
  attr!(
    supports_idle_tasks,
    JsRuntimeAttributeEnableIdleProcessing,
    "Enable idle processing. `run_idle_tasks` must be called regularly."
  );

  /// Set the runtime's memory limit.
  pub fn memory_limit(mut self, limit: usize) -> Self {
    self.memory_limit = Some(limit);
    self
  }

  /// Set a callback for when the runtime collects garbage.
  pub fn collect_callback(mut self, callback: Box<CollectCallback>) -> Self {
    self.collect_callback = Some(callback);
    self
  }

  /// Creates the runtime object with associated settings.
  pub fn build(self) -> Result<Runtime> {
    let mut handle = JsRuntimeHandle::new();
    jstry(unsafe { JsCreateRuntime(self.attributes, None, &mut handle) })?;

    if let Some(limit) = self.memory_limit {
      jstry(unsafe { JsSetRuntimeMemoryLimit(handle, limit) })?;
    }

    let callback = self.collect_callback.map(|callback| unsafe {
      let mut wrapper = Box::new(callback);
      jsassert!(JsSetRuntimeBeforeCollectCallback(
        handle,
        &mut *wrapper as *mut _ as *mut _,
        Some(Runtime::before_collect)
      ));
      wrapper
    });

    Ok(Runtime {
      last_idle: None,
      last_idle_tick: None,
      handle,
      callback,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{script, test, Context, Runtime};
  use std::{
    sync::{Arc, Mutex},
    thread,
  };

  #[test]
  fn minimal() {
    test::run_with_context(|guard| {
      let result = script::eval(guard, "5 + 5").unwrap();
      assert_eq!(result.to_integer(guard), 10);
    });
  }

  #[test]
  fn collect_callback() {
    let called = Arc::new(Mutex::new(false));
    {
      let called = called.clone();
      let runtime = Runtime::builder()
        .collect_callback(Box::new(move || *called.lock().unwrap() = true))
        .build()
        .unwrap();
      runtime.collect().unwrap();
    }
    assert!(*called.lock().unwrap());
  }

  #[test]
  fn thread_send() {
    let runtime = Runtime::new().unwrap();
    let context = Context::new(&runtime).unwrap();
    let result = {
      let guard = context.make_current().unwrap();
      script::eval(&guard, "[5, 'foo', {}]")
        .unwrap()
        .into_array()
        .unwrap()
    };

    thread::spawn(move || {
      let guard = context.make_current().unwrap();
      assert_eq!(result.len(&guard), 3);
    })
    .join()
    .unwrap();
  }
}
