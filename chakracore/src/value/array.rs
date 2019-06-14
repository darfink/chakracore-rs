use libc::c_void;
use std::{mem, ptr, slice};
use chakracore_sys::*;
use boolinator::Boolinator;
use context::ContextGuard;
use super::{Value, Object};
use Property;

/// A JavaScript array.
pub struct Array(JsValueRef);

/// An iterator for a JavaScript array.
pub struct ArrayIter<'a> {
    guard: &'a ContextGuard<'a>,
    array: &'a Array,
    index: u32,
    size: u32,
}

/// A JavaScript array buffer.
pub struct ArrayBuffer(JsValueRef);

impl Array {
    /// Creates a new array with a specified length.
    pub fn new(_guard: &ContextGuard, length: u32) -> Self {
        let mut reference = JsValueRef::new();
        unsafe {
            jsassert!(JsCreateArray(length, &mut reference));
            Self::from_raw(reference)
        }
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
            Self::from_raw(reference)
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
            Self::from_raw(buffer)
        }
    }

    /// Creates a new array buffer, wrapping external data.
    ///
    /// This is unsafe because the object does not take ownership of the
    /// resource. Therefore the data may become a dangling pointer. The caller is
    /// responsible for keeping the reference alive.
    pub unsafe fn from_slice<T: Sized>(_guard: &ContextGuard, data: &mut [T]) -> Self {
        let base = data.as_mut_ptr() as *mut _;
        let size = (data.len() * mem::size_of::<T>()) as _;

        let mut buffer = JsValueRef::new();
        jsassert!(JsCreateExternalArrayBuffer(base, size, None, ptr::null_mut(), &mut buffer));
        Self::from_raw(buffer)
    }

    /// Returns the underlying memory storage used by the array buffer.
    ///
    /// This may produce unexpected results if used in conjunction with the
    /// unsafe `from_slice`.
    pub fn as_slice(&self) -> &[u8] {
        let mut data = ptr::null_mut();
        let mut size = 0;
        unsafe {
            jsassert!(JsGetArrayBufferStorage(self.as_raw(), &mut data, &mut size));
            slice::from_raw_parts(data, size as usize)
        }
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
        (self.index < self.size).as_some_from(|| {
            self.index += 1;
            self.array.get_index(self.guard, self.index - 1)
        })
    }
}

reference!(Array);
inherit!(Array, Object);
subtype!(Array, Value);
reference!(ArrayBuffer);
inherit!(ArrayBuffer, Object);
subtype!(ArrayBuffer, Value);

#[cfg(test)]
mod tests {
    use {test, value};

    #[test]
    fn iterator() {
        test::run_with_context(|guard| {
            let length = 10;
            let array = value::Array::new(guard, length);

            for i in 0..length {
                array.set_index(guard, i, value::Number::new(guard, i as i32));
            }

            assert_eq!(array.len(guard), 10);
            assert_eq!(array.iter(guard).fold(0, |acc, value| acc + value.to_integer(guard)), 45);
        });
    }

    #[test]
    fn buffer_storage() {
        test::run_with_context(|guard| {
            let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

            let array = value::ArrayBuffer::with_data(guard, data.clone());
            assert_eq!(array.as_slice(), data.as_slice());

            let array = unsafe { value::ArrayBuffer::from_slice(guard, &mut data) };
            assert_eq!(array.as_slice(), data.as_slice());
        });
    }
}
