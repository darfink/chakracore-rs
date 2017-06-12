/// Equivalent to the normal `try!` macro for JSRT function calls.
macro_rules! jstry {
    ($e: expr) => {
        match $e {
            ::chakracore_sys::JsErrorCode::NoError => (),
            error @ _ => return Err(format!("JSRT call failed with: {:?}", error).into()),
        }
    }
}

/// Asserts the return value of a JSRT function call.
macro_rules! jsassert {
    ($e: expr, $name: expr) => {
        let result = $e;

        // In many cases idiomatic code prevents any errors from happening
        // (except for memory resource issues).
        assert!(result == ::chakracore_sys::JsErrorCode::NoError,
                format!("Call to '{}' failed with: {:?}", $name, result));
    };

    ($e: expr) => {
        jsassert!($e, stringify!($e));
    };
}

/// Implements JSRT reference counting for a type.
macro_rules! reference {
    ($typ:ident) => {
        impl $typ {
            /// Creates an instance from a raw pointer.
            ///
            /// This is used for managing the lifetime of JSRT objects. They are
            /// tracked using reference counting; incrementing with `from_raw`,
            /// and decrementing with `drop`.
            ///
            /// This is required to support items stored on the heap, since the
            /// JSRT runtime only observes the stack.
            pub unsafe fn from_raw(value: JsRef) -> $typ {
                jsassert!(JsAddRef(value, ::std::ptr::null_mut()));
                $typ(value)
            }
        }

        impl Clone for $typ {
            fn clone(&self) -> $typ {
                unsafe { $typ::from_raw(self.as_raw()) }
            }
        }

        impl Drop for $typ {
            /// Decrements the reference's counter.
            fn drop(&mut self) {
                let mut count = 0;
                jsassert!(unsafe { JsRelease(self.as_raw(), &mut count) });
                debug_assert!(count < ::libc::c_uint::max_value());
            }
        }
    }
}

/// Implements a relationship between two subtypes.
macro_rules! subtype {
    ($child:ident, $parent:ident) => {
        impl From<$child> for $parent {
            fn from(child: $child) -> $parent {
                unsafe { ::std::mem::transmute(child) }
            }
        }
    }
}

/// Implements an inheritance between two types.
macro_rules! inherit {
    ($child:ident, $parent:ident) => {
        subtype!($child, $parent);

        impl ::std::ops::Deref for $child {
            type Target = $parent;

            fn deref(&self) -> &Self::Target {
                unsafe { ::std::mem::transmute(self) }
            }
        }
    }
}

/// Used for JavaScript value type implementation.
macro_rules! is_same {
    ($target:ident, $target_doc:expr) => {
        #[doc=$target_doc]
        pub fn is_same(value: &Value) -> bool {
            value.get_type() == JsValueType::$target
        }
    };
}
