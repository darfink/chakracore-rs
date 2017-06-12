/// Equivalent to the normal `try!` macro for JSRT function calls.
macro_rules! jstry {
    ($e: expr) => { ::util::jstry($e)?; }
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

/// Shared base reference implementation.
macro_rules! reference_base {
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
            /// Duplicates a reference counted type.
            ///
            /// The underlying pointer will be copied, and its reference count
            /// will be incremented, returned wrapped as the type.
            fn clone(&self) -> $typ {
                unsafe { $typ::from_raw(self.as_raw()) }
            }
        }
    }
}

/// Implements JSRT reference counting for a non-value type.
macro_rules! reference {
    ($typ:ident) => {
        reference_base!($typ);

        impl Drop for $typ {
            /// Decrements the reference counter.
            fn drop(&mut self) {
                ::util::release_reference(self.as_raw());
            }
        }
    };
}
