## NOTE: This project is not currently maintained, due to ChakraCore itself is no longer being actively developed.

<div align="center">

# `chakracore-rs`

[![crates.io version][crate-shield]][crate]
[![Documentation][docs-shield]][docs]
[![Language (Rust)][rust-shield]][rust]

</div>

**chakracore-rs is an iditiomatic wrapper for
[ChakraCore](https://github.com/Microsoft/ChakraCore), written in Rust.**

This repository contains two crates:
- [chakracore-sys](#chakracore-sys) - raw bindings to the JavaScript
Runtime.
- [chakracore](#chakracore) - an idiomatic wrapper, built on the
chakracore-sys crate.

## chakracore

This is a wrapper around the [JavaScript Runtime (JSRT)](https://goo.gl/1F6Gi1),
used in [Microsoft Edge](https://www.microsoft.com/en-us/windows/microsoft-edge)
and [node-chakracore](https://github.com/nodejs/node-chakracore). The library is
still in pre-release and is not yet stable. The tests try to cover as much
functionality as possible but memory leaks and segfaults may occur. If you want
a more stable library, use the underlying API directly;
[chakracore-sys](#chakracore-sys).

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
chakracore = "0.2"
```

... and this to your crate root:

```rust
extern crate chakracore as js;
```

*NOTE: See additional build instructions for [chakracore-sys](#chakracore-sys)*

### Examples

#### Hello World

```rust
extern crate chakracore as js;

fn main() {
  let runtime = js::Runtime::new().unwrap();
  let context = js::Context::new(&runtime).unwrap();
  let guard = context.make_current().unwrap();

  let result = js::script::eval(&guard, "5 + 5").unwrap();
  assert_eq!(result.to_integer(&guard), 10);
}
```

#### Function - Multiply

```rust
extern crate chakracore as js;

fn main() {
  let runtime = js::Runtime::new().unwrap();
  let context = js::Context::new(&runtime).unwrap();
  let guard = context.make_current().unwrap();

  let multiply = js::value::Function::new(&guard, Box::new(|guard, info| {
      let result = info.arguments[0].to_integer(guard)
                 * info.arguments[1].to_integer(guard);
      Ok(js::value::Number::new(guard, result).into())
  }));

  let result = multiply.call(&guard, &[
      &js::value::Number::new(&guard, 191).into(),
      &js::value::Number::new(&guard, 7).into(),
  ]).unwrap();

  assert_eq!(result.to_integer(&guard), 1337);
}
```

## chakracore-sys

This library handles the static and dynamic linking of the JavaScript
Runtime. The rust bindings are generated (on the fly) for the interface,
therefore the entire API is exposed and accessable.

A *Hello World* example can be found in
[src/lib.rs](./chakracore-sys/src/lib.rs).

An example of the generated bindings can be found
[here](https://gist.github.com/darfink/d519756ad88efcddfbfe895439cf9451).

### Requirements

This library builds the ChakraCore component in the source tree. It is cloned
by the build script and built in test-mode (same as release, but includes
more runtime checks). If custom build settings are desired, ChakraCore can be
built manually, out of tree, and specified using two environment variables:

* `CHAKRA_SOURCE`: The root of the ChakraCore checkout.
* `CHAKRA_BUILD`: The `bin` directory of the build.
  - Default on Windows: `%CHAKRA_SOURCE%\Build\VcBuild\bin\{BUILD_TYPE}`.
  - Default on Unix: `$CHAKRA_SOURCE/BuildLinux/{BUILD_TYPE}`.

This script has not been tested with the `--embed-icu` option.

#### Static/Shared

By default, this library links ChakraCore dynamically. There is a feature
called `static` that builds it by linking to the generated archive instead.
On windows, only shared library builds are available as of this time (see
[#279](https://github.com/Microsoft/ChakraCore/issues/279)).

#### Prerequisites

The library naturally shares all of ChakraCore's dependencies. Beyond this,
[rust-bindgen](https://github.com/servo/rust-bindgen) is used in the build
script, which requires `clang-3.8` or later. On Unix `pkg-config` is required as
well.

##### Windows

* Visual Studio 2013/2015/2017 with:
  - Windows SDK 8.1
  - C++ support
* `clang-3.8` or later. Downloads can be found
  [here](http://llvm.org/releases/download.html).  
  Remember to add LLVM directories to `PATH` during installation.
* Rust MSVC toolchain (i.e `rustup install stable-msvc`).  
  This is required since ChakraCore uses the MSVC ABI.
* If building for ARM: [Windows 10 SDK (July
  2015)](https://developer.microsoft.com/en-us/windows/downloads/sdk-archive)

##### macOS

```
$ brew install cmake icu4c llvm38 pkg-config
```

##### Debian-based linuxes

```
# apt-get install -y build-essential cmake clang libunwind8-dev \
#     libicu-dev llvm-3.8-dev libclang-3.8-dev pkg-config liblzma-dev
```

#### Building

- ##### Windows

  Ensure that you are running in a Visual Studio command line environment,
  either by sourcing `vcvarsall.bat`, or by building from the Visual
  Studio Command Prompt.

  ```
  $ cargo test -vv
  ```

- ##### Unix

  ```
  $ cargo test -vv [--features static]
  ```

In case you find yourself stuck in the build process, open an
[issue](https://github.com/darfink/chakracore-rs/issues/new).

#### Status

This library has been built on `macOS 10.12 x86_64`, `Ubuntu 16.10 x86_64` and
`Windows 10 x86_x64`.
<!-- Links -->
[crate-shield]: https://img.shields.io/crates/v/chakracore.svg?style=flat-square
[crate]: https://crates.io/crates/chakracore
[rust-shield]: https://img.shields.io/badge/powered%20by-rust-blue.svg?style=flat-square
[rust]: https://www.rust-lang.org
[docs-shield]: https://img.shields.io/badge/docs-crates-green.svg?style=flat-square
[docs]: https://darfink.github.io/chakracore-rs/chakracore/index.html
