use libc::c_void;
use jsrt_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;
use PropertyId;

/// Callback type for collector.
type BeforeCollectCallback = Fn(&Value);

/// A JavaScript object.
#[derive(Clone, Debug)]
pub struct Object(JsValueRef);

// TODO: Add support for finalize callback
impl Object {
    /// Creates a new empty object.
    pub fn new(_guard: &ContextGuard) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateObject(&mut value));
            Object::from_raw(value)
        }
    }

    /// Creates a new object with external data.
    ///
    /// The object takes ownership of the resource. It is undetermined when, and
    /// even if, the destructor is called. It relies on the engine's finalize
    /// callback.
    ///
    /// As long as the object is referenced on the stack or in any script
    /// context, the `external` data will be kept alive (i.e it is not tied to
    /// the handle).
    pub fn with_external<T>(_guard: &ContextGuard, external: Box<T>) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            // TODO: Move this to a custom `External` type.
            jsassert!(JsCreateExternalObject(Box::into_raw(external) as *mut _,
                                             Some(Self::finalize::<T>),
                                             &mut value));
            Object::from_raw(value)
        }
    }

    /// Creates an object from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Object(reference)
    }

    /// Sets an object's property's value.
    pub fn set(&self, _guard: &ContextGuard, key: &PropertyId, value: &Value) {
        jsassert!(unsafe { JsSetProperty(self.as_raw(), key.as_raw(), value.as_raw(), false) });
    }

    /// Sets an object's index value.
    pub fn set_index(&self, guard: &ContextGuard, index: u32, value: &Value) {
        let index = super::Number::new(guard, index as i32);
        jsassert!(unsafe { JsSetIndexedProperty(self.as_raw(), index.as_raw(), value.as_raw()) });
    }

    /// Returns an object's property's value.
    pub fn get(&self, _guard: &ContextGuard, key: &PropertyId) -> Value {
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
    pub fn delete(&self, _guard: &ContextGuard, key: &PropertyId) -> bool {
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
    pub fn has(&self, _guard: &ContextGuard, key: &PropertyId) -> bool {
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
    pub fn define_property(&self,
                           _guard: &ContextGuard,
                           key: &PropertyId,
                           descriptor: &Object)
                           -> bool {
        let mut result = false;
        jsassert!(unsafe {
            JsDefineProperty(self.as_raw(),
                             key.as_raw(),
                             descriptor.as_raw(),
                             &mut result)
        });
        result
    }

    /// Sets the object's prototype.
    pub fn set_prototype(&self, _guard: &ContextGuard, prototype: &Value) -> Result<()> {
        unsafe {
            jstry!(JsSetPrototype(self.as_raw(), prototype.as_raw()));
            Ok(())
        }
    }

    /// Returns the object's prototype.
    pub fn get_prototype(&self, _guard: &ContextGuard) -> Result<Value> {
        let mut prototype = JsValueRef::new();
        unsafe {
            jstry!(JsGetPrototype(self.as_raw(), &mut prototype));
            Ok(Value::from_raw(prototype))
        }
    }

    /// Returns whether the object has external data or not.
    pub fn has_external_data(&self) -> bool {
        let mut result = false;
        unsafe {
            jsassert!(JsHasExternalData(self.as_raw(), &mut result));
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

    /// Sets a callback that is executed before object is collected.
    ///
    /// This is highly unsafe to use. There is no bookkeeping whether another
    /// caller replaces the current callback or not. It is also used internally
    /// by `Function` to cleanup user data (if it's replaced, memory will leak).
    pub unsafe fn set_collect_callback(&self, callback: Box<BeforeCollectCallback>) {
        let wrapper = Box::new(callback);
        let api = JsSetObjectBeforeCollectCallback;
        jsassert!(api(self.as_raw(),
                      Box::into_raw(wrapper) as *mut _,
                      Some(Self::collect)));
    }

    /// A collect callback, triggered before the object is destroyed.
    unsafe extern "system" fn collect(value: JsValueRef, data: *mut c_void) {
        let wrapper: Box<Box<BeforeCollectCallback>> = Box::from_raw(data as *mut _);
        wrapper(&Value::from_raw(value));
    }

    /// A finalizer callback, triggered before an external is removed.
    unsafe extern "system" fn finalize<T>(data: *mut c_void) {
        Box::from_raw(data as *mut T);
    }
}

inherit!(Object, Value);
