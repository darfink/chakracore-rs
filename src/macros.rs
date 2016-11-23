macro_rules! jstry {
    ($e: expr) => {
        match $e {
            ::chakracore_sys::JsErrorCode::NoError => (),
            error @ _ => return Err(format!("JSRT call failed with {:?}", error).into()),
        }
    }
}

macro_rules! jsassert {
    ($e: expr) => {
        // In some cases idiomatic code should prevent any errors from
        // happening (except for memory resource issues).
        assert!($e == ::chakracore_sys::JsErrorCode::NoError,
                concat!("Call to '", stringify!($e), "' failed"))
    }
}

macro_rules! subtype {
    ($child:ident, $parent:ident) => {
        impl From<$child> for $parent {
            fn from(child: $child) -> $parent {
                unsafe { ::std::mem::transmute(child) }
            }
        }
    }
}

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

macro_rules! is_same {
    ($target:ident, $target_doc:expr) => {
        #[doc=$target_doc]
        pub fn is_same(value: &Value) -> bool {
            value.get_type() == JsValueType::$target
        }
    };
}
