# `chakracore-sys`

This is a library for the [JavaScript Runtime (JSRT)](https://goo.gl/1F6Gi1), an
API used for embedding Microsoft's ChakraCore into applications. This library
handles static and dynamic linking of the runtime, and generates rust
bindings (on the fly) for the interface. The entire API is generated and
accessable.

A *Hello World* example can be found in
[src/lib.rs](https://github.com/darfink/chakracore-rs/blob/master/chakracore-sys/src/lib.rs).

An example of the generated bindings can be found
[here](https://gist.github.com/darfink/d519756ad88efcddfbfe895439cf9451).

If you are interested in idiomatic Rust bindings, check out
[chakracore-rs](https://github.com/darfink/chakracore-rs).

**NOTE: The version on crates.io (version `0.0.2`) does not include support for
generating bindings on-the-fly. This is because of the `libbindgen`
dependecy, which is not yet published on crates.io. If this functionality is
desired, use the git repository instead.

## Requirements

This library builds the ChakraCore component in the source tree. It is cloned by
the build script and built in test-mode (same as release, but includes more
runtime checks). The current version used is `1.4`. It has also been tested with
versions `1.2` and `1.3`. If custom build settings are desired, ChakraCore can
be built manually, out of tree, and specified using two environment variables:

* `CHAKRA_SOURCE`: The root of the ChakraCore checkout.
* `CHAKRA_BUILD`: The `bin` directory of the build.
  - Default on Windows: `%CHAKRA_SOURCE%\Build\VcBuild\bin\{BUILD_TYPE}`.
  - Default on Unix: `$CHAKRA_SOURCE/BuildLinux/{BUILD_TYPE}`.

This script has not been tested with the `--embed-icu` option.

### Static/Shared

By default, this library links ChakraCore dynamically. There is a feature called
`static` that builds it by linking to the three generated archives instead. On
windows, only shared library builds are available as of this time. See
[#279](https://github.com/Microsoft/ChakraCore/issues/279)

### Prerequisites

The library naturally shares all of ChakraCore's dependencies. Beyond this,
[rust-bindgen](https://github.com/servo/rust-bindgen) is used in the build
script, which requires `clang-3.8` or later. On Unix `pkg-config` is required as
well.

#### Windows

* Visual Studio 2013 or 2015 with C++ support.
* `clang-3.8` or later. Downloads can be found
  [here](http://llvm.org/releases/download.html).  
  Remember to add LLVM directories to `PATH` during installation.
* Rust MSVC toolchain (i.e `rustup install stable-msvc`).  
  This is required since ChakraCore uses the MSVC ABI.
* If building for ARM: [Windows 10 SDK (July
  2015)](https://developer.microsoft.com/en-us/windows/downloads/sdk-archive)

#### macOS

```
# brew install cmake icu4c llvm38 pkg-config
```

If you choose to install `icu4c` (required for ChakraCore) using Brew, you need
make `pkg-config` aware of the library. This is because Brew does not link the
library with the system, as it may conflict with other builds. There are two
possible solutions to this.

- Forcefully link the library with the system:

  ```
  # brew link icu4c --force
  ```

- Or, before you build the library, export `icu4c`'s package configuration:

  ```
  # export PKG_CONFIG_PATH="$(brew --prefix)/opt/icu4c/lib/pkgconfig"
  ```

#### On Debian-based linuxes

```
# apt-get install -y build-essential cmake clang libunwind-dev \
#     libicu-dev llvm-3.8-dev libclang-3.8-dev pkg-config liblzma-dev
```

### Building

- ##### Windows

  Ensure that you are running in a Visual Studio command line environment,
  either by sourcing `vcvarsall.bat`, or by building from the Visual
  Studio Command Prompt.

  ```
  # cargo test -vv
  ```

- ##### Unix

  ```
  # cargo test -vv [--features static]
  ```

When you run the build, there should be no *missing variable* warnings.

In case you use a custom ChakraCore build using `CHAKRA_SOURCE/BUILD`, remember
that if an environment variable is changed *after* running the build script, you
need to recompile it:

```
# cargo clean -p chakracore-sys && cargo build [--features static]
```

In case you find yourself stuck in the build process, open an
[issue](https://github.com/darfink/chakracore-rs/issues/new).

### Status

This library has been built on `macOS 10.12 x86_64`, `Ubuntu 16.10 x86_64` and
`Windows 10 x86_x64`.
