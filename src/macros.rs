macro_rules! jstry {
    ($e: expr) => {
        match $e {
            // TODO: Add support for stack traces
            ::chakra_sys::JsErrorCode::NoError => (),
            error @ _ => return Err(format!("ChakraCore call failed with {:?}", error).into()),
        }
    }
}

macro_rules! jsassert {
    ($e: expr) => {
        // In some cases idiomatic code should prevent any errors from
        // happening (except for memory resource issues).
        assert_eq!($e, ::chakra_sys::JsErrorCode::NoError)
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
