use std::fmt;
use chakracore_sys::*;
use context::{Context, ContextGuard};

/// A property identifier used with objects.
pub struct Property(JsPropertyIdRef);

impl Property {
    /// Creates a property identifier from a string.
    ///
    /// If a property identifier with this name has already been created, it
    /// will return it instead of creating a new one.
    ///
    /// Reuse the property identifier objects as much as possible. For each
    /// constructor call, the string is copied and converted to UTF-16.
    pub fn new(_guard: &ContextGuard, name: &str) -> Self {
        let bytes = name.as_bytes();
        let mut reference = JsPropertyIdRef::new();
        unsafe {
            jsassert!(JsCreatePropertyId(bytes.as_ptr() as _, bytes.len(), &mut reference));
            Self::from_raw(reference)
        }
    }

    /// Converts a JavaScript property to a native string.
    pub fn to_string(&self, _guard: &ContextGuard) -> String {
        ::util::to_string_impl(self.as_raw(), JsCopyPropertyId).unwrap()
    }

    /// Returns the underlying raw pointer behind this property.
    pub fn as_raw(&self) -> JsPropertyIdRef {
        self.0
    }
}

impl fmt::Debug for Property {
    /// Only use for debugging, it relies on an implicit active context and uses
    /// several unwraps.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard = unsafe { Context::get_current().unwrap() };
        let output = self.to_string(&guard);
        write!(f, "Property('{}')", output)
    }
}

reference!(Property);

#[cfg(test)]
mod tests {
    use {test, Property};

    #[test]
    fn string_conversion() {
        test::run_with_context(|guard| {
            let property = Property::new(guard, "foo");
            assert_eq!(property.to_string(guard), "foo");
        });
    }
}