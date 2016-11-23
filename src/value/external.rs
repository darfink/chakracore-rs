use std::ptr;
use libc::c_void;
use chakracore_sys::*;
use context::ContextGuard;
use super::{Value, Object};

/// A JavaScript external object.
#[derive(Clone)]
pub struct External(JsValueRef);

// TODO: Make the entire implementation generic
impl External {
    /// Creates a new object with external data.
    ///
    /// The object takes ownership of the resource. It is undetermined when, and
    /// even if, the destructor is called. It relies on the engine's finalize
    /// callback.
    ///
    /// As long as the object is referenced on the stack or in any script
    /// context, the `external` data will be kept alive (i.e it is not tied to
    /// the handle).
    pub fn new<T>(_guard: &ContextGuard, external: Box<T>) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateExternalObject(Box::into_raw(external) as *mut _,
                                            Some(Self::finalize::<T>),
                                            &mut value));
            External::from_raw(value)
        }
    }

    /// Creates an external object from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        External(reference)
    }

    /// Returns the external object's data.
    pub unsafe fn value<'a, T>(&'a self) -> &'a T {
        let mut data = ptr::null_mut();
        jsassert!(JsGetExternalData(self.as_raw(), &mut data));
        (data as *mut T).as_ref().unwrap()
    }

    /// Returns true if the value is an `External`.
    pub fn is_same(value: &Value) -> bool {
        value.get_type() == JsValueType::Object && Self::has_external_data(value)
    }

    /// Returns whether the value has external data or not.
    fn has_external_data(value: &Value) -> bool {
        let mut result = false;
        jsassert!(unsafe { JsHasExternalData(value.as_raw(), &mut result) });
        result
    }

    /// A finalizer callback, triggered before an external is removed.
    unsafe extern "system" fn finalize<T>(data: *mut c_void) {
        Box::from_raw(data as *mut T);
    }
}

inherit!(External, Object);
subtype!(External, Value);
