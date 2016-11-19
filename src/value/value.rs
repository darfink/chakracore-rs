use std::{fmt, mem, ptr};
use chakra_sys::*;
use context::{Context, ContextGuard};
use error::*;

macro_rules! downcast {
    ($predicate:ident, $predicate_doc:expr, $result:ident) => {
        #[doc=$predicate_doc]
        pub fn $predicate(&self) -> bool {
            self.get_type().unwrap() == JsValueType::$result
        }
    };
    ($predicate:ident, $predicate_doc:expr,
     $conversion:ident, $conversion_doc:expr, $result:ident) => {
        downcast!($predicate, $predicate_doc, $result);

        #[doc=$conversion_doc]
        pub fn $conversion(self) -> Option<super::$result> {
            if self.$predicate() {
                Some(unsafe { mem::transmute(self) })
            } else {
                None
            }
        }
    };
}

macro_rules! nativecast {
    ($name:ident, $name_doc:expr, $result:ident, $into:ident, $represent:ident, $native:ident) => {
        #[doc=$name_doc]
        pub fn $name(&self, _guard: &ContextGuard) -> Result<$result> {
            match self.clone().$into() {
                None => self.$represent(_guard)?,
                Some(value) => value,
            }.$native()
        }
    }
}

macro_rules! representation {
    ($name:ident, $name_doc:expr, $result:ident, $function:ident) => {
        #[doc=$name_doc]
        pub fn $name(&self, _guard: &ContextGuard) -> Result<super::$result> {
            let mut value = JsValueRef::new();
            unsafe {
                jstry!($function(self.as_raw(), &mut value));
                Ok(super::$result::from_raw(value))
            }
        }

    }
}

#[derive(Clone)]
pub struct Value(JsValueRef);

impl Value {
    /// Creates a value from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Value(reference)
    }

    // Converts a value to another custom type
    downcast!(is_string,
              "Returns true if this value is a `String`.",
              into_string,
              "Converts the value to a `String`.",
              String);
    downcast!(is_number,
              "Returns true if this value is a `Number`.",
              into_number,
              "Converts the value to a `Number`.",
              Number);
    downcast!(is_boolean,
              "Returns true if this value is a `Boolean`.",
              into_boolean,
              "Converts the value to a `Boolean`.",
              Boolean);

    // Converts a value to a native type
    nativecast!(
        to_string_convert,
        "Transforms the value to a native string, containing the value's string representation.",
        String, into_string, string_representation, to_string);
    nativecast!(
        to_integer_convert,
        "Transforms the value to a native string, containing the value's integer representation.",
        i32, into_number, number_representation, to_integer);
    nativecast!(
        to_double_convert,
        "Transforms the value to a native `f64`, containing the value's floating point representation.",
        f64, into_number, number_representation, to_double);
    nativecast!(
        to_boolean_convert,
        "Transforms the value to a native boolean, containing the value's bool representation.",
        bool, into_boolean, boolean_representation, to_bool);

    // Converts a value to the JavaScript expression of another type
    representation!(
        boolean_representation,
        "Converts the value to its `JavaScript` boolean representation.",
        Boolean, JsConvertValueToBoolean);
    representation!(
        string_representation,
        "Converts the value to its `JavaScript` string representation.",
        String, JsConvertValueToString);
    representation!(
        number_representation,
        "Converts the value to its `JavaScript` number representation.",
        Number, JsConvertValueToNumber);

    /// Creates an array buffer, wrapping external data.
    pub unsafe fn from_external_slice<T: Sized>(_guard: &ContextGuard,
                                                data: &mut [T]) -> Result<Value> {
        let base = data.as_mut_ptr() as *mut _;
        let size = (data.len() * mem::size_of::<T>()) as usize as _;

        let mut buffer = JsValueRef::new();
        jstry!(JsCreateExternalArrayBuffer(base, size, None, ptr::null_mut(), &mut buffer));
        Ok(Value(buffer))
    }

    /// Returns the type of the value.
    pub fn get_type(&self) -> Result<JsValueType> {
        let mut value_type = JsValueType::Undefined;
        jstry!(unsafe { JsGetValueType(self.as_raw(), &mut value_type) });
        Ok(value_type)
    }

    /// Returns the underlying raw pointer.
    pub fn as_raw(&self) -> JsValueRef {
        self.0
    }
}

impl fmt::Debug for Value {
    /// Only use for debugging, it relies on an implicit active context and uses
    /// several unwraps.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard = unsafe { Context::get_current().unwrap() };
        let output = self.to_string_convert(&guard).unwrap();

        let value_type = self.get_type().unwrap();
        match value_type {
            JsValueType::String => write!(f, "Value({:?}: '{}')", value_type, output),
            _ => write!(f, "Value({:?}: {})", value_type, output),
        }
    }
}
