use std::{mem, ptr};
use chakra_sys::*;
use context::ContextGuard;
use super::{Value, Object};

/// A JavaScript array.
#[derive(Clone, Debug)]
pub struct Array(JsValueRef);

/// An instance of an array buffer.
#[derive(Clone, Debug)]
pub struct ArrayBuffer(JsValueRef);

impl Array {
    /// Creates a new array with a specified length.
    pub fn new(_guard: &ContextGuard, length: u32) -> Self {
        let mut reference = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateArray(length, &mut reference));
            Array::from_raw(reference)
        }
    }

    /// Creates an array from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        Array(reference)
    }
}

impl ArrayBuffer {
    /// Creates a new array buffer with a specified size.
    pub fn new(_guard: &ContextGuard, size: u32) -> Self {
        let mut reference = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateArrayBuffer(size, &mut reference));
            ArrayBuffer::from_raw(reference)
        }
    }

    /// Creates an array buffer, wrapping external data.
    pub unsafe fn from_slice<T: Sized>(_guard: &ContextGuard, data: &mut [T]) -> ArrayBuffer {
        let base = data.as_mut_ptr() as *mut _;
        let size = (data.len() * mem::size_of::<T>()) as usize as _;

        // TODO: Use the finalize callback
        let mut buffer = JsValueRef::new();
        jsassert!(JsCreateExternalArrayBuffer(base, size, None, ptr::null_mut(), &mut buffer));
        ArrayBuffer::from_raw(buffer)
    }

    /// Creates an array from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        ArrayBuffer(reference)
    }
}

inherit!(Array, Object);
subtype!(Array, Value);
inherit!(ArrayBuffer, Object);
subtype!(ArrayBuffer, Value);
