# `jsrt-rs`

This is a wrapper around the [JavaScript Runtime (JSRT)](https://goo.gl/1F6Gi1),
used in [Microsoft Edge](https://www.microsoft.com/en-us/windows/microsoft-edge)
and [node-chakracore](https://github.com/nodejs/node-chakracore). The library is
still in pre-release and is not yet stable. The tests try to cover as much
functionality as possible but memory leaks and segfaults may occur. If you want
a more stable library, use the underlying API directly with
[jsrt-sys](https://github.com/darfink/jsrt-rs/tree/master/jsrt-sys).

## Documentation

https://docs.rs/jsrt

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
jsrt = "0.1.0"
```

and this to your crate root:

```rust
extern crate jsrt;
```

This library, by itself is simple and easily installed, but its `jsrt-sys`
dependency is *not*. To ensure a successful build, please view the `jsrt-sys`
[build
instructions](https://github.com/darfink/jsrt-rs/tree/master/jsrt-sys#prerequisites).

## Example

### Hello World

```rust
extern crate jsrt;

fn main() {
  let runtime = jsrt::Runtime::new().unwrap();
  let context = jsrt::Context::new(&runtime).unwrap();
  let guard = context.make_current().unwrap();

  let result = jsrt::Script::run(&guard, "(5 + 5)").unwrap();
  assert_eq!(result.to_integer_convert(&guard), 10);
}
```

### Function - Multiply

```rust
extern crate jsrt;

fn main() {
  let runtime = jsrt::Runtime::new().unwrap();
  let context = jsrt::Context::new(&runtime).unwrap();
  let guard = context.make_current().unwrap();

  let multiply = jsrt::value::Function(&guard, Box::new(|guard, info| {
      let result = info.arguments[0].to_integer_convert(guard)
                 * info.arguments[1].to_integer_convert(guard);
      Ok(jsrt::value::Number::new(guard, result).into())
  });

  let result = multiply.call(&guard, &jsrt::value::null(&guard), &[
      jsrt::value::Number::new(&guard, 191).into(),
      jsrt::value::Number::new(&guard, 7).into(),
  ]).unwrap();

  assert_eq!(result.to_integer_convert(&guard), 1337);
}
```