#![cfg_attr(feature = "unstable", feature(test))]
//! A library for interfacing with ChakraCore using the JSRT API.
//!
//! This crate provides abstractions over nearly every JSRT API available, in a
//! thread- and memory-safe implementation.
//!
//! ```rust
//! extern crate chakracore as js;
//!
//! fn main() {
//!   let runtime = js::Runtime::new().unwrap();
//!   let context = js::Context::new(&runtime).unwrap();
//!   let guard = context.make_current().unwrap();
//!
//!   let result = js::script::eval(&guard, "(5 + 5)").unwrap();
//!   assert_eq!(result.to_integer(&guard), 10);
//! }
//! ```
//!
//! *NOTE: During pre-release (0.X.X) stability may vary.*

#[cfg(test)]
#[macro_use]
extern crate matches;

extern crate anymap;
extern crate boolinator;
extern crate chakracore_sys;
extern crate libc;

pub use context::Context;
pub use error::{Error, Result};
pub use property::Property;
pub use runtime::Runtime;

#[macro_use]
mod macros;
mod error;
mod property;
mod util;
pub mod runtime;
pub mod context;
pub mod script;
pub mod value;

#[cfg(test)]
mod test {
    use super::*;

    pub fn setup_env() -> (Runtime, Context) {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();
        (runtime, context)
    }

    pub fn run_with_context<T: FnOnce(&context::ContextGuard)>(callback: T) {
        let (_runtime, context) = setup_env();
        context.exec_with(callback).unwrap();
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use self::test::Bencher;
    use super::*;

    fn setup_env() -> (Runtime, Context) {
        let runtime = Runtime::new().unwrap();
        let context = Context::new(&runtime).unwrap();
        (runtime, context)
    }

    #[bench]
    fn property_bench(bench: &mut Bencher) {
        let (_runtime, context) = setup_env();

        let guard = context.make_current().unwrap();
        let object = value::Object::new(&guard);
        object.set(&guard, Property::new(&guard, "test"), value::Number::new(&guard, 10));

        bench.iter(|| {
            (0..10000).fold(0, |acc, _| acc + object.get(&guard, Property::new(&guard, "test")).to_integer(&guard));
        });
    }
}
