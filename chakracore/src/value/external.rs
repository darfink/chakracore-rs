use crate::context::ContextGuard;
use crate::value::{Object, Value};
use chakracore_sys::*;
use libc::c_void;
use std::ptr;

/// A JavaScript external object.
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
      jsassert!(JsCreateExternalObject(
        Box::into_raw(external) as *mut _,
        Some(Self::finalize::<T>),
        &mut value
      ));
      Self::from_raw(value)
    }
  }

  /// Creates a new object with external data.
  ///
  /// This is unsafe because the object does not take ownership of the
  /// resource. Therefore the data may become a dangling pointer. The caller
  /// is responsible for keeping the reference alive.
  pub unsafe fn from_ptr<T>(_guard: &ContextGuard, external: *mut T) -> Self {
    let mut value = JsValueRef::new();
    jsassert!(JsCreateExternalObject(external as *mut _, None, &mut value));
    Self::from_raw(value)
  }

  /// Returns the external object's data.
  pub unsafe fn value<T>(&self) -> &mut T {
    let mut data = ptr::null_mut();
    jsassert!(JsGetExternalData(self.as_raw(), &mut data));
    (data as *mut T).as_mut().expect("retrieving external data")
  }

  /// Returns true if the value is an `External`.
  pub fn is_same<V: AsRef<Value>>(value: V) -> bool {
    value.as_ref().get_type() == JsValueType::Object && Self::has_external_data(value.as_ref())
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

reference!(External);
inherit!(External, Object);
subtype!(External, Value);

#[cfg(test)]
mod tests {
  use crate::{test, value};

  #[test]
  fn destructor() {
    static mut CALLED: bool = false;
    {
      struct Foo(i32);
      impl Drop for Foo {
        fn drop(&mut self) {
          assert_eq!(self.0, 10);
          unsafe { CALLED = true };
        }
      }

      test::run_with_context(|guard| {
        let _ = value::External::new(guard, Box::new(Foo(10)));
      });
    }
    assert!(unsafe { CALLED });
  }
}
