use chakra_sys::*;
use context::ContextGuard;
use error::*;
use Context;
use super::Value;

/// Callback type for destructors.
//pub type FinalizeCallback = Fn(&ContextGuard, &CallbackInfo) -> ::std::result::Result<Value, Value> + 'static;

#[derive(Clone, Debug)]
pub struct Object(JsValueRef);

impl Object {
    /// Creates a new object.
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

    /// Returns the `Context` associated with the object.
    pub fn get_context(&self) -> Result<Context> {
        let mut context = JsContextRef::new();
        unsafe {
            jstry!(JsGetContextOfObject(self.as_raw(), &mut context));
            Ok(Context::from_raw(context))
        }
    }
}

inherit!(Object, Value);
