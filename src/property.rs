use std::fmt;
use error::*;
use jsrt_sys::*;
use context::{Context, ContextGuard};
use value;

/// A property identifier used with objects.
#[derive(Clone)]
pub struct PropertyId(JsPropertyIdRef);

// TODO: Add heiarchy and support for symbols
impl PropertyId {
    /// Creates a property identifier from a string.
    ///
    /// If a property identifier with this name has already been created, it
    /// will return it instead of creating a new one.
    ///
    /// Reuse property identifiers as much as possible. For constructor call,
    /// the string is copied and converted to UTF-16.
    pub fn from_str(_guard: &ContextGuard, name: &str) -> PropertyId {
        let bytes = name.as_bytes();
        let mut reference = JsPropertyIdRef::new();
        unsafe {
            assert_eq!(JsCreatePropertyIdUtf8(bytes.as_ptr() as _, bytes.len(), &mut reference),
                       JsErrorCode::NoError);
            PropertyId::from_raw(reference)
        }
    }

    /// Creates a property identifier from a raw pointer.
    pub unsafe fn from_raw(reference: JsPropertyIdRef) -> PropertyId {
        PropertyId(reference)
    }

    /// Returns an object's associated context.
    pub unsafe fn from_object(object: &value::Object) -> Result<Context> {
        let mut reference = JsContextRef::new();
        jstry!(JsGetContextOfObject(object.as_raw(), &mut reference));
        Ok(Context::from_raw(reference))
    }

    /// Converts a JavaScript property to a native string.
    pub fn to_string(&self, _guard: &ContextGuard) -> Result<String> {
        ::util::to_string_impl(self.as_raw(), JsCopyPropertyIdUtf8)
    }

    /// Returns the underlying raw pointer behind this property.
    pub fn as_raw(&self) -> JsPropertyIdRef {
        self.0
    }
}

impl fmt::Debug for PropertyId {
    /// Only use for debugging, it relies on an implicit active context and uses
    /// several unwraps.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard = unsafe { Context::get_current().unwrap() };
        let output = self.to_string(&guard).unwrap();
        write!(f, "Property('{}')", output)
    }
}
