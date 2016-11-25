use libc::c_void;
use std::{mem, ptr};
use chakracore_sys::*;
use context::ContextGuard;
use super::{Value, Object};
use Property;

/// A JavaScript array.
#[derive(Clone)]
pub struct Array(JsValueRef);

/// An iterator for a JavaScript array.
pub struct ArrayIter<'a> {
    guard: &'a ContextGuard<'a>,
    array: &'a Array,
    index: u32,
    size: u32,
}

/// A JavaScript array buffer.
#[derive(Clone)]
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

    /// Returns the length of the array.
    pub fn len(&self, guard: &ContextGuard) -> usize {
        let length = Property::new(&guard, "length");
        self.get(guard, &length).to_integer(&guard) as usize
    }

    /// Returns an iterator for the array.
    pub fn iter<'a>(&'a self, guard: &'a ContextGuard) -> ArrayIter<'a> {
        ArrayIter {
            guard: guard,
            size: self.len(guard) as u32,
            array: self,
            index: 0,
        }
    }

    is_same!(Array, "Returns true if the value is an `Array`.");
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

    /// Creates a new array buffer, owning the data.
    pub fn with_data<T: Sized>(_guard: &ContextGuard, data: Vec<T>) -> Self {
        let mut data = Box::new(data);
        let base = data.as_mut_ptr() as *mut _;
        let size = data.len() * mem::size_of::<T>();

        unsafe {
            let mut buffer = JsValueRef::new();
            jsassert!(JsCreateExternalArrayBuffer(base,
                                                  size as _,
                                                  Some(Self::finalize::<T>),
                                                  Box::into_raw(data) as *mut _,
                                                  &mut buffer));
            ArrayBuffer::from_raw(buffer)
        }
    }

    /// Creates a new array buffer, wrapping external data.
    pub unsafe fn from_slice<T: Sized>(_guard: &ContextGuard, data: &mut [T]) -> ArrayBuffer {
        let base = data.as_mut_ptr() as *mut _;
        let size = (data.len() * mem::size_of::<T>()) as _;

        let mut buffer = JsValueRef::new();
        jsassert!(JsCreateExternalArrayBuffer(base, size, None, ptr::null_mut(), &mut buffer));
        ArrayBuffer::from_raw(buffer)
    }

    /// Creates an array from a raw pointer.
    pub unsafe fn from_raw(reference: JsValueRef) -> Self {
        ArrayBuffer(reference)
    }

    is_same!(ArrayBuffer, "Returns true if the value is an `ArrayBuffer`.");

    /// A finalizer callback, triggered before an external buffer is removed.
    unsafe extern "system" fn finalize<T>(data: *mut c_void) {
        Box::from_raw(data as *mut Vec<T>);
    }
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = Value;

    /// Returns the next element in the array.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            self.index += 1;
            Some(self.array.get_index(self.guard, self.index - 1))
        } else {
            None
        }
    }
}

inherit!(Array, Object);
subtype!(Array, Value);
inherit!(ArrayBuffer, Object);
subtype!(ArrayBuffer, Value);
