use chakra_sys::*;
use context::ContextGuard;
use error::*;
use super::Value;
use PropertyId;

/// Callback type for destructors.
//pub type FinalizeCallback = Fn(&ContextGuard, &CallbackInfo) -> ::std::result::Result<Value, Value> + 'static;

#[derive(Clone, Debug)]
pub struct Object(JsValueRef);

impl Object {
    /// Creates a new empty object.
    pub fn new(_guard: &ContextGuard) -> Self {
        let mut value = JsValueRef::new();
        unsafe {
            assert_eq!(JsCreateObject(&mut value), JsErrorCode::NoError);
            Object::from_raw(value)
        }
    }

    //pub unsafe fn with_external<T>(external: &mut T, Option<C>)

    /// Creates an object from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Object(reference)
    }

    /// Sets an object's key's value.
    pub fn set(&self, _guard: &ContextGuard, key: &PropertyId, value: &Value) -> Result<()> {
        jstry!(unsafe { JsSetProperty(self.as_raw(), key.as_raw(), value.as_raw(), false) });
        Ok(())
    }

    /// Sets an object's index value.
    pub fn set_index(&self, guard: &ContextGuard, index: u32, value: &Value) -> Result<()> {
        let index = super::Number::new(guard, index as i32);
        jstry!(unsafe { JsSetIndexedProperty(self.as_raw(), index.as_raw(), value.as_raw()) });
        Ok(())
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
            assert_eq!(JsHasExternalData(self.as_raw(), &mut result), JsErrorCode::NoError);
            result
        }
    }
}

inherit!(Object, Value);
