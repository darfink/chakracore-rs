# `jsrt-sys`

This is a library for the [JavaScript Runtime (JSRT)](https://goo.gl/1F6Gi1), an
API used for embedding Microsoft's ChakraCore into applications. This library
handles static and/or dynamic linking of the runtime, and generates rust
bindings (on the fly) for the interface. The entire API is generated and
accessable (except for the functions used for debugging, those are not yet
available on Unix OSes).

A *Hello World* example can be found in
[src/lib.rs](https://github.com/darfink/jsrt-rs/blob/master/jsrt-sys/src/lib.rs).

An example of the generated bindings can be found
[here](https://gist.github.com/darfink/d519756ad88efcddfbfe895439cf9451).

If you are interested in idiomatic Rust bindings, check out
[jsrt-rs](https://github.com/darfink/jsrt-rs).

## Requirements

Before being able to use this library, ChakraCore needs to be built. It is a
rather complex build process and the script is not stable, so this library does
not automate it (yet). Look
[here](https://github.com/Microsoft/ChakraCore/wiki/Building-ChakraCore) for
build instructions. This library has been tested with the 1.3 release and
latest [master](https://github.com/Microsoft/ChakraCore/commit/446b086d17).

The build script uses two environment variables to find the required files.

- `CHAKRA_SOURCE`: Should point to root of the ChakraCore checkout.
- `CHAKRA_BUILD`: Should point to the build directory of ChakraCore.  
By default it is `$CHAKRA_SOURCE/Build(Linux)/{BUILD_TYPE}`.

This script has not been tested with the `--embed-icu` option.

### Static/Shared

By default, this library links ChakraCore statically. There is a feature called
shared that builds it by linking to `libChakraCore.so` instead.

### Prerequisites

Besides the dependencies for ChakraCore (cmake, clang-3.7, ICU), it also uses
Servo's [rust-bindgen](https://github.com/servo/rust-bindgen), which requires
clang-3.8 or later. The build script also heavily relies on pkg-config.

**NOTE:** The following instructions assume you already have ChakraCore's
 dependencies installed.

#### macOS

```
# brew install llvm38 pkg-config
```

If you installed ICU4C (required for ChakraCore) using Brew, and wish to link
statically, you need make pkg-config aware of the library. This is because Brew
does not link this library with the system, it may conflict with other builds.
There are two possible solutions to this.

- Forcefully link the library with the system:

  ```
  # brew link icu4c --force
  ```

- Or, before you build this library, export ICU4C's package configuration:

  ```
  export PKG_CONFIG_PATH=/usr/local/opt/icu4c/lib/pkgconfig
  ```

#### On Debian-based linuxes

```
# apt-get install llvm-3.8-dev libclang-3.8-dev pkg-config
```

### Building

After building ChakraCore and installing all dependencies, prepare the build by
telling the script where the ChakraCore files can be found.

```
# export CHAKRA_SOURCE=/path/to/chakracore/checkout
# export CHAKRA_BUILD=/path/to/chakracore/build/directory
# cargo build -vv && cargo test
```

Remember that if you change the environment variables *after* running the build
script, you need to recompile it.

```
cargo clean -p jsrt-sys && cargo build
```

In case you find yourself stuck in the build process, open an
[issue](https://github.com/darfink/jsrt-rs/issues/new).

### Status

This library has been built on `macOS 10.12 x86_64` and `Ubuntu 16.10 x86_64`.