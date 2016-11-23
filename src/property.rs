use std::fmt;
use chakracore_sys::*;
use context::{Context, ContextGuard};

/// A property identifier used with objects.
#[derive(Clone)]
pub struct Property(JsPropertyIdRef);

impl Property {
    /// Creates a property identifier from a string.
    ///
    /// If a property identifier with this name has already been created, it
    /// will return it instead of creating a new one.
    ///
    /// Reuse the property identifier objects as much as possible. For each
    /// constructor call, the string is copied and converted to UTF-16.
    pub fn from_str(_guard: &ContextGuard, name: &str) -> Self {
        let bytes = name.as_bytes();
        let mut reference = JsPropertyIdRef::new();
        unsafe {
            jsassert!(JsCreatePropertyIdUtf8(bytes.as_ptr() as _, bytes.len(), &mut reference));
            Property::from_raw(reference)
        }
    }

    /// Creates a property identifier from a raw pointer.
    pub unsafe fn from_raw(reference: JsPropertyIdRef) -> Self {
        Property(reference)
    }

    /// Converts a JavaScript property to a native string.
    pub fn to_string(&self, _guard: &ContextGuard) -> String {
        ::util::to_string_impl(self.as_raw(), JsCopyPropertyIdUtf8).unwrap()
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
