/// Implements JSRT reference counting for a value type (overrides default
/// `reference` macro definition).
macro_rules! reference {
  ($typ:ident) => {
    reference_base!($typ);

    impl Drop for $typ {
      /// Decrements the reference counter if the object is recyclable.
      fn drop(&mut self) {
        use {util, Context};
        Context::exec_with_value(self, |_| {
          // This requires that the active context is the same as the
          // one it was created with (this is not mentioned whatsoever
          // in the ChakraCore documentation).
          util::release_reference(self.as_raw());
        })
        .expect("changing active context for release");
      }
    }
  };
}

/// Implements a relationship between two subtypes.
macro_rules! subtype {
  ($child:ident, $parent:ident) => {
    impl From<$child> for $parent {
      fn from(child: $child) -> $parent {
        unsafe { ::std::mem::transmute(child) }
      }
    }

    impl AsRef<$parent> for $child {
      fn as_ref(&self) -> &$parent {
        unsafe { ::std::mem::transmute(self) }
      }
    }
  };
}

/// Implements inheritance between two types.
macro_rules! inherit {
  ($child:ident, $parent:ident) => {
    subtype!($child, $parent);

    impl ::std::ops::Deref for $child {
      type Target = $parent;

      fn deref(&self) -> &Self::Target {
        unsafe { ::std::mem::transmute(self) }
      }
    }
  };
}

/// Implements JavaScript type equality method.
macro_rules! is_same {
  ($target:ident, $target_doc:expr) => {
    #[doc=$target_doc]
    pub fn is_same<V: AsRef<Value>>(value: V) -> bool {
      value.as_ref().get_type() == JsValueType::$target
    }
  };
}
