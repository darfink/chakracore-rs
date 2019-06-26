use crate::value::{Array, Function, Value};
use crate::{util::jstry, ContextGuard, Property, Result};
use chakracore_sys::*;
use libc::c_void;

/// Callback type for collector.
type BeforeCollectCallback = dyn Fn(&Value);

/// A JavaScript object.
pub struct Object(JsValueRef);

// TODO: Add `for .. in` iterator
impl Object {
  /// Creates a new empty object.
  pub fn new(_guard: &ContextGuard) -> Self {
    let mut value = JsValueRef::new();
    unsafe {
      jsassert!(JsCreateObject(&mut value));
      Self::from_raw(value)
    }
  }

  /// Sets an object's property's value.
  pub fn set<P, V>(&self, _guard: &ContextGuard, key: P, value: V)
  where
    P: AsRef<Property>,
    V: AsRef<Value>,
  {
    jsassert!(unsafe {
      JsSetProperty(
        self.as_raw(),
        key.as_ref().as_raw(),
        value.as_ref().as_raw(),
        false,
      )
    });
  }

  /// Sets an object's index value.
  pub fn set_index<V: AsRef<Value>>(&self, guard: &ContextGuard, index: u32, value: V) {
    let index = super::Number::new(guard, index as i32);
    jsassert!(unsafe {
      JsSetIndexedProperty(self.as_raw(), index.as_raw(), value.as_ref().as_raw())
    });
  }

  /// Returns an object's property's value.
  pub fn get<P: AsRef<Property>>(&self, _guard: &ContextGuard, key: P) -> Value {
    let mut result = JsValueRef::new();
    unsafe {
      jsassert!(JsGetProperty(
        self.as_raw(),
        key.as_ref().as_raw(),
        &mut result
      ));
      Value::from_raw(result)
    }
  }

  /// Returns an object's index value.
  pub fn get_index(&self, guard: &ContextGuard, index: u32) -> Value {
    let index = super::Number::new(guard, index as i32);
    let mut result = JsValueRef::new();
    unsafe {
      jsassert!(JsGetIndexedProperty(
        self.as_raw(),
        index.as_raw(),
        &mut result
      ));
      Value::from_raw(result)
    }
  }

  /// Deletes an object's property.
  pub fn delete<P: AsRef<Property>>(&self, _guard: &ContextGuard, key: P) -> bool {
    let mut result = JsValueRef::new();
    unsafe {
      jsassert!(JsDeleteProperty(
        self.as_raw(),
        key.as_ref().as_raw(),
        false,
        &mut result
      ));
      super::Boolean::from_raw(result).value()
    }
  }

  /// Deletes an object's index.
  pub fn delete_index(&self, guard: &ContextGuard, index: u32) {
    let index = super::Number::new(guard, index as i32);
    jsassert!(unsafe { JsDeleteIndexedProperty(self.as_raw(), index.as_raw()) });
  }

  /// Determines whether an object has a property.
  pub fn has<P: AsRef<Property>>(&self, _guard: &ContextGuard, key: P) -> bool {
    let mut result = false;
    jsassert!(unsafe { JsHasProperty(self.as_raw(), key.as_ref().as_raw(), &mut result) });
    result
  }

  /// Determines whether an object has a value at the specified index.
  pub fn has_index(&self, guard: &ContextGuard, index: u32) -> bool {
    let mut result = false;
    let index = super::Number::new(guard, index as i32);
    jsassert!(unsafe { JsHasIndexedProperty(self.as_raw(), index.as_raw(), &mut result) });
    result
  }

  /// Defines or modifies a property directly on an object.
  ///
  /// This is equivalent to `Object.defineProperty()`.
  pub fn define_property<P, O>(&self, _guard: &ContextGuard, key: P, desc: O) -> bool
  where
    P: AsRef<Property>,
    O: AsRef<Object>,
  {
    let mut result = false;
    jsassert!(unsafe {
      JsDefineProperty(
        self.as_raw(),
        key.as_ref().as_raw(),
        desc.as_ref().as_raw(),
        &mut result,
      )
    });
    result
  }

  /// Sets the object's prototype. This will result in an error if it's called
  /// on the context's global object.
  pub fn set_prototype<V: AsRef<Value>>(&self, _guard: &ContextGuard, prototype: V) -> Result<()> {
    unsafe { jstry(JsSetPrototype(self.as_raw(), prototype.as_ref().as_raw())) }
  }

  /// Returns the object's prototype.
  pub fn get_prototype(&self, _guard: &ContextGuard) -> Value {
    let mut prototype = JsValueRef::new();
    unsafe {
      jsassert!(JsGetPrototype(self.as_raw(), &mut prototype));
      Value::from_raw(prototype)
    }
  }

  /// Returns the object's property names
  pub fn get_own_property_names(&self, _guard: &ContextGuard) -> Array {
    let mut properties = JsValueRef::new();
    unsafe {
      jsassert!(JsGetOwnPropertyNames(self.as_raw(), &mut properties));
      Array::from_raw(properties)
    }
  }

  /// Returns whether the object is an instance of this `Function` or not.
  ///
  /// This must only be used on values that exists within the same context as
  /// the constructor, otherwise the result will always be `false`.
  pub fn instance_of<F: AsRef<Function>>(&self, _guard: &ContextGuard, constructor: F) -> bool {
    let mut result = false;
    // TODO: #[cfg(debug_assertions)] validate same context
    unsafe {
      jsassert!(JsInstanceOf(
        self.as_raw(),
        constructor.as_ref().as_raw(),
        &mut result
      ));
      result
    }
  }

  /// Makes an object non-extensible.
  pub fn prevent_extension(&self) {
    jsassert!(unsafe { JsPreventExtension(self.as_raw()) });
  }

  /// Returns whether the object is extensible or not.
  pub fn is_extensible(&self) -> bool {
    let mut result = false;
    jsassert!(unsafe { JsGetExtensionAllowed(self.as_raw(), &mut result) });
    result
  }

  /// Sets a callback that is executed before the object is collected.
  ///
  /// This is highly unsafe to use. There is no bookkeeping whether any other
  /// caller replaces the current callback or not. It is also used internally
  /// by `Function` to cleanup user data (if it's replaced, memory will leak).
  pub unsafe fn set_collect_callback(&self, callback: Box<BeforeCollectCallback>) {
    let wrapper = Box::new(callback);
    let api = JsSetObjectBeforeCollectCallback;
    jsassert!(api(
      self.as_raw(),
      Box::into_raw(wrapper) as *mut _,
      Some(Self::collect)
    ));
  }

  /// Returns true if the value is an `Object`.
  pub fn is_same<V: AsRef<Value>>(value: V) -> bool {
    match value.as_ref().get_type() {
      JsValueType::Object
      | JsValueType::Function
      | JsValueType::Error
      | JsValueType::Array
      | JsValueType::ArrayBuffer
      | JsValueType::TypedArray
      | JsValueType::DataView => true,
      _ => false,
    }
  }

  /// A collect callback, triggered before the object is destroyed.
  unsafe extern "system" fn collect(value: JsValueRef, data: *mut c_void) {
    let wrapper: Box<Box<BeforeCollectCallback>> = Box::from_raw(data as *mut _);
    wrapper(&Value::from_raw(value));
  }
}

reference!(Object);
inherit!(Object, Value);

#[cfg(test)]
mod tests {
  use crate::{script, test, value, Context, Property, Runtime};

  #[test]
  fn properties() {
    test::run_with_context(|guard| {
      let object = value::Object::new(guard);

      // Associate it with an object field
      let prop_foo = Property::new(guard, "foo");
      let prop_bar = Property::new(guard, "bar");

      object.set(guard, &prop_foo, value::Number::new(guard, 10));
      object.set(guard, &prop_bar, value::null(guard));

      // Ensure the fields have been created with the assigned values
      assert_eq!(object.get(guard, &prop_foo).to_integer(guard), 10);
      assert!(object.get(guard, &prop_bar).is_null());

      // Retrieve all the objects' properties
      let properties = object
        .get_own_property_names(guard)
        .iter(guard)
        .map(|val| val.to_string(guard))
        .collect::<Vec<_>>();
      assert_eq!(properties, ["foo", "bar"]);

      // Remove the object's property
      assert!(object.has(guard, &prop_foo));
      object.delete(guard, &prop_foo);
      assert!(!object.has(guard, &prop_foo));
    });
  }

  #[test]
  fn instance_of() {
    test::run_with_context(|guard| {
      let constructor = value::Function::new(
        guard,
        Box::new(move |_, info| {
          assert!(info.is_construct_call);
          Ok(info.this)
        }),
      );

      let global = guard.global();
      let property = Property::new(guard, "FooBar");
      global.set(guard, property, &constructor);

      let foo_bar = script::eval(guard, "new FooBar()")
        .unwrap()
        .into_object()
        .unwrap();
      assert!(foo_bar.instance_of(guard, &constructor));
    });
  }

  #[test]
  fn instance_of_cross_contexts() {
    let runtime = Runtime::new().unwrap();
    let c1 = Context::new(&runtime).unwrap();
    let c2 = Context::new(&runtime).unwrap();

    let g1 = c1.make_current().unwrap();
    let p1 = script::eval(&g1, "new Promise(() => true)")
      .ok()
      .and_then(|v| v.into_object())
      .unwrap();

    let g2 = c2.make_current().unwrap();
    let p2 = script::eval(&g1, "Promise")
      .ok()
      .and_then(|v| v.into_function())
      .unwrap();
    assert!(p1.instance_of(&g2, &p2));
  }
}
