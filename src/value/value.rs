use std::{fmt, mem};
use chakracore_sys::*;
use context::{Context, ContextGuard};
use value;

macro_rules! downcast {
    ($predicate:ident, $predicate_doc:expr, $target:ident) => {
        #[doc=$predicate_doc]
        pub fn $predicate(&self) -> bool {
            value::$target::is_same(self)
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
        pub fn $name(&self, _guard: &ContextGuard) -> $result {
            match self.clone().$into() {
                None => self.$represent(_guard),
                Some(value) => value,
            }.$native()
        }
    }
}

macro_rules! representation {
    ($name:ident, $name_doc:expr, $result:ident, $function:ident) => {
        #[doc=$name_doc]
        pub fn $name(&self, _guard: &ContextGuard) -> super::$result {
            let mut value = JsValueRef::new();
            unsafe {
                jsassert!($function(self.as_raw(), &mut value));
                super::$result::from_raw(value)
            }
        }

    }
}

/// A JavaScript value, base class for all types.
///
/// All values are tied to a specific context and should not be reused in
/// between.
///
/// The underlying object is represented as a `JsValueRef`, a reference to a
/// ChakraCore value.
///
/// This type implements the `Debug` trait, but it should be used carefully. It
/// assumes there is an active context (the same context that the value was
/// created with).
///
/// Do not get intimidated by all the conversion methods. They are very easy to
/// grok — there are three different types:
///
/// > `into_*`
/// >> These do not modify any data. They only check the type of the
/// underlying value. If the value is the designated type (e.g `Object`), the
/// underlying pointer is copied and returned wrapped as the specific type.
///
/// > `*_representation`
/// >> These create a new value, by casting to a specific type using JavaScript
/// semantics. For example; calling `number_representation` on an `Object`
/// results in a `Number(NaN)`. Casting a `Boolean(false)` using
/// `string_representation` results in a `String('false')`.
///
/// > `to_*`
/// >> These are utility functions to easily retrieve a native representation of
/// the internal value. The chain of actions performed is the following: `into_*() ->
/// [*_representation()] -> value()`. A call to `*_representation` is only
/// performed if required (i.e a string is not redundantly converted to a
/// string).
pub struct Value(JsValueRef);

impl Value {
    // Transforms a value to another custom type
    downcast!(is_undefined,
              "Returns true if this value is `undefined`.",
              Undefined);
    downcast!(is_null, "Returns true if this value is `null`.", Null);
    downcast!(is_number,
              "Returns true if this value is a `Number`.",
              into_number,
              "Represent the value as a `Number`. Does not affect the underlying value.",
              Number);
    downcast!(is_string,
              "Returns true if this value is a `String`.",
              into_string,
              "Represent the value as a `String`. Does not affect the underlying value.",
              String);
    downcast!(is_boolean,
              "Returns true if this value is a `Boolean`.",
              into_boolean,
              "Represent the value as a `Boolean`. Does not affect the underlying value.",
              Boolean);
    downcast!(is_object,
              "Returns true if this value is an `Object`.",
              into_object,
              "Represent the value as an `Object`. Does not affect the underlying value.",
              Object);
    downcast!(is_external,
              "Returns true if this value is an `External`.",
              into_external,
              "Represent the value as an `External`. Does not affect the underlying value.",
              External);
    downcast!(is_function,
              "Returns true if this value is a `Function`.",
              into_function,
              "Represent the value as a `Function`. Does not affect the underlying value.",
              Function);
    downcast!(is_array,
              "Returns true if this value is an `Array`.",
              into_array,
              "Represent the value as an `Array`. Does not affect the underlying value.",
              Array);
    downcast!(is_array_buffer,
              "Returns true if this value is an `ArrayBuffer`.",
              into_array_buffer,
              "Represent the value as an `ArrayBuffer`. Does not affect the underlying value.",
              ArrayBuffer);

    // Converts a value to a native type
    nativecast!(to_string,
                "Converts the value to a native string, containing the value's string representation.",
                String,
                into_string,
                string_representation,
                value);
    nativecast!(to_integer,
                "Converts the value to a native integer, containing the value's integer representation.",
                i32,
                into_number,
                number_representation,
                value);
    nativecast!(to_double,
                "Converts the value to a native double, containing the value's floating point representation.",
                f64,
                into_number,
                number_representation,
                value_double);
    nativecast!(to_bool,
                "Converts the value to a native boolean, containing the value's bool representation.",
                bool,
                into_boolean,
                boolean_representation,
                value);

    /// Converts the value to a native string, containing the value's JSON representation.
    pub fn to_json(&self, guard: &ContextGuard) -> String {
        // TODO: Use native functionality when implemented
        let stringify = ::script::eval(guard, "JSON.stringify")
            .expect("JSON.stringify does not exist")
            .into_function()
            .expect("JSON.stringify is not a function");
        stringify.call(guard, &[self]).unwrap().to_string(guard)
    }

    // Casts a value to the JavaScript expression of another type
    representation!(boolean_representation,
                    "Creates a new boolean with this value represented as `Boolean`.",
                    Boolean,
                    JsConvertValueToBoolean);
    representation!(number_representation,
                    "Creates a new number with this value represented as `Number`.",
                    Number,
                    JsConvertValueToNumber);
    representation!(object_representation,
                    "Creates a new object with this value represented as `Object`.",
                    Object,
                    JsConvertValueToObject);
    representation!(string_representation,
                    "Creates a new string with this value represented as `String`.",
                    String,
                    JsConvertValueToString);

    /// Returns the type of the value. This method should be used with
    /// consideration. It does not keep track of custom types, such as
    /// `External`. It only returns the runtime's definition of a type.
    pub fn get_type(&self) -> JsValueType {
        let mut value_type = JsValueType::Undefined;
        jsassert!(unsafe { JsGetValueType(self.as_raw(), &mut value_type) });
        value_type
    }

    /// Compare two values for equality (`==`).
    pub fn equals(&self, _guard: &ContextGuard, that: &Value) -> bool {
        let mut result = false;
        jsassert!(unsafe { JsEquals(self.as_raw(), that.as_raw(), &mut result) });
        result
    }

    /// Compare two values for strict equality (`===`).
    pub fn strict_equals(&self, _guard: &ContextGuard, that: &Value) -> bool {
        let mut result = false;
        jsassert!(unsafe { JsStrictEquals(self.as_raw(), that.as_raw(), &mut result) });
        result
    }

    /// Returns the underlying raw pointer.
    pub fn as_raw(&self) -> JsValueRef {
        self.0
    }
}

impl PartialEq for Value {
    /// Use sparingly (prefer `equals`), this relies on an implicitly active context.
    fn eq(&self, other: &Value) -> bool {
        let guard = unsafe { Context::get_current().unwrap() };
        self.strict_equals(&guard, other)
    }
}

impl fmt::Debug for Value {
    /// Only use for debugging, it relies on an implicitly active context.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let guard = unsafe { Context::get_current().unwrap() };
        let output = self.to_string(&guard);

        let value_type = self.get_type();
        match value_type {
            JsValueType::String => write!(f, "Value({:?}: '{}')", value_type, output),
            _ => write!(f, "Value({:?}: {})", value_type, output),
        }
    }
}

reference!(Value);

#[cfg(test)]
mod tests {
    use {test, value, Property};

    #[test]
    fn json_conversion() {
        test::run_with_context(|guard| {
            let object = value::Object::new(guard);
            object.set(guard, &Property::new(guard, "foo"), &value::Number::new(guard, 1337));
            assert_eq!(object.to_json(guard), r#"{"foo":1337}"#);
        });
    }
}
