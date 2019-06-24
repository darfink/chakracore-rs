use super::Value;
use chakracore_sys::*;
use context::ContextGuard;

/// A JavaScript boolean.
pub struct Boolean(JsValueRef);

impl Boolean {
  /// Creates a new boolean.
  pub fn new(_guard: &ContextGuard, boolean: bool) -> Self {
    let mut value = JsValueRef::new();
    unsafe {
      jsassert!(JsBoolToBoolean(boolean, &mut value));
      Self::from_raw(value)
    }
  }

  /// Converts a JavaScript boolean to a bool.
  pub fn value(&self) -> bool {
    let mut boolean = false;
    jsassert!(unsafe { JsBooleanToBool(self.as_raw(), &mut boolean) });
    boolean
  }

  is_same!(Boolean, "Returns true if the value is a `Boolean`.");
}

reference!(Boolean);
inherit!(Boolean, Value);
