//! A JavaScript object and associated types.
use libc::c_void;
use chakracore_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;
use Property;

/// Callback type for collector.
type BeforeCollectCallback = Fn(&Value);

/// A JavaScript object.
#[derive(Clone)]
pub struct Object(JsValueRef);

impl Object {
    /// Creates a new empty object.
    pub fn new(_guard: &ContextGuard) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateObject(&mut value));
            Object::from_raw(value)
        }
    }

    /// Creates an object from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Object(reference)
    }

    /// Sets an object's property's value.
    pub fn set(&self, _guard: &ContextGuard, key: &Property, value: &Value) {
        jsassert!(unsafe { JsSetProperty(self.as_raw(), key.as_raw(), value.as_raw(), false) });
    }

    /// Sets an object's index value.
    pub fn set_index(&self, guard: &ContextGuard, index: u32, value: &Value) {
        let index = super::Number::new(guard, index as i32);
        jsassert!(unsafe { JsSetIndexedProperty(self.as_raw(), index.as_raw(), value.as_raw()) });
    }

    /// Returns an object's property's value.
    pub fn get(&self, _guard: &ContextGuard, key: &Property) -> Value {
        let mut result = JsValueRef::new();
        unsafe {
            jsassert!(JsGetProperty(self.as_raw(), key.as_raw(), &mut result));
            Value::from_raw(result)
        }
    }

    /// Returns an object's index value.
    pub fn get_index(&self, guard: &ContextGuard, index: u32) -> Value {
        let index = super::Number::new(guard, index as i32);
        let mut result = JsValueRef::new();
        unsafe {
            jsassert!(JsGetIndexedProperty(self.as_raw(), index.as_raw(), &mut result));
            Value::from_raw(result)
        }
    }

    /// Deletes an object's property.
    pub fn delete(&self, _guard: &ContextGuard, key: &Property) -> bool {
        let mut result = JsValueRef::new();
        unsafe {
            jsassert!(JsDeleteProperty(self.as_raw(), key.as_raw(), false, &mut result));
            super::Boolean::from_raw(result).value()
        }
    }

    /// Deletes an object's index.
    pub fn delete_index(&self, guard: &ContextGuard, index: u32) {
        let index = super::Number::new(guard, index as i32);
        jsassert!(unsafe { JsDeleteIndexedProperty(self.as_raw(), index.as_raw()) });
    }

    /// Determines whether an object has a property.
    pub fn has(&self, _guard: &ContextGuard, key: &Property) -> bool {
        let mut result = false;
        jsassert!(unsafe { JsHasProperty(self.as_raw(), key.as_raw(), &mut result) });
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
    pub fn define_property(&self, _guard: &ContextGuard, key: &Property, desc: &Object) -> bool {
        let mut result = false;
        jsassert!(unsafe {
            JsDefineProperty(self.as_raw(), key.as_raw(), desc.as_raw(), &mut result)
        });
        result
    }

    /// Sets the object's prototype. This will result in an error if it's called
    /// on the context's global object.
    pub fn set_prototype(&self, _guard: &ContextGuard, prototype: &Value) -> Result<()> {
        unsafe {
            jstry!(JsSetPrototype(self.as_raw(), prototype.as_raw()));
            Ok(())
        }
    }

    /// Returns the object's prototype.
    pub fn get_prototype(&self, _guard: &ContextGuard) -> Value {
        let mut prototype = JsValueRef::new();
        unsafe {
            jsassert!(JsGetPrototype(self.as_raw(), &mut prototype));
            Value::from_raw(prototype)
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
        jsassert!(api(self.as_raw(),
                      Box::into_raw(wrapper) as *mut _,
                      Some(Self::collect)));
    }

    /// Returns true if the value is an `Object`.
    pub fn is_same(value: &Value) -> bool {
        match value.get_type() {
            JsValueType::Object      |
            JsValueType::Function    |
            JsValueType::Error       |
            JsValueType::Array       |
            JsValueType::ArrayBuffer |
            JsValueType::TypedArray  |
            JsValueType::DataView    => true,
            _ => false,
        }
    }

    /// A collect callback, triggered before the object is destroyed.
    unsafe extern "system" fn collect(value: JsValueRef, data: *mut c_void) {
        let wrapper: Box<Box<BeforeCollectCallback>> = Box::from_raw(data as *mut _);
        wrapper(&Value::from_raw(value));
    }
}

inherit!(Object, Value);
