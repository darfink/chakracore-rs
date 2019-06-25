use crate::{Context, ContextGuard};
use chakracore_sys::*;
use std::fmt;

/// A property identifier used with objects.
#[derive(PartialEq)]
pub struct Property(JsPropertyIdRef);

impl Property {
  /// Creates a property identifier from a string.
  ///
  /// If a property identifier with this name has already been created, it will
  /// be returned instead of creating a new one.
  pub fn new(_guard: &ContextGuard, name: &str) -> Self {
    let bytes = name.as_bytes();
    let mut reference = JsPropertyIdRef::new();
    unsafe {
      jsassert!(JsCreatePropertyId(
        bytes.as_ptr() as _,
        bytes.len(),
        &mut reference
      ));
      Self::from_raw(reference)
    }
  }

  /// Converts a JavaScript property to a native string.
  pub fn to_string(&self, _guard: &ContextGuard) -> String {
    crate::util::to_string_impl(self.as_raw(), JsCopyPropertyId)
      .expect("converting property to string")
  }
}

impl fmt::Debug for Property {
  /// Only use for debugging, it relies on an implicit active context and
  /// panics otherwise.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    Context::exec_with_current(|guard| {
      let output = self.to_string(&guard);
      write!(f, "Property('{}')", output)
    })
    .expect("property debug output without an active context")
  }
}

reference!(Property);

#[cfg(test)]
mod tests {
  use crate::{test, Property};

  #[test]
  fn string_conversion() {
    test::run_with_context(|guard| {
      let property = Property::new(guard, "foo");
      assert_eq!(property.to_string(guard), "foo");
    });
  }
}
