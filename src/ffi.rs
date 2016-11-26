use std::ptr;
use std::sync::{Once, ONCE_INIT};
use libc::{c_int, c_void};

#[link(name = "ChakraCore")]
extern "system" {
    pub fn DllMain(instance: *mut c_void, reason: usize, reserved: *mut c_void) -> c_int;
}

static START: Once = ONCE_INIT;

pub fn initialize() {
    START.call_once(|| unsafe {
        // This is required on Unix platforms
        DllMain(ptr::null_mut(), 1, ptr::null_mut());
        DllMain(ptr::null_mut(), 2, ptr::null_mut());
    });
}
