# `chakracore-rs`

This is a wrapper around the [JavaScript Runtime (JSRT)](https://goo.gl/1F6Gi1),
used in [Microsoft Edge](https://www.microsoft.com/en-us/windows/microsoft-edge)
and [node-chakracore](https://github.com/nodejs/node-chakracore). The library is
still in pre-release and is not yet stable. The tests try to cover as much
functionality as possible but memory leaks and segfaults may occur. If you want
a more stable library, use the underlying API directly with
[chakracore-sys](https://github.com/darfink/chakracore-rs/tree/master/chakracore-sys).

## Documentation

https://docs.rs/chakracore

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
chakracore = "0.1.0"
```

and this to your crate root:

```rust
extern crate chakracore as js;
```

This library, by itself is simple and easily installed, but its
`chakracore-sys` dependency is *not*. To ensure a successful build, please view
the `chakracore-sys` [build
instructions](https://github.com/darfink/chakracore-rs/tree/master/chakracore-sys#requirements).

## Example

### Hello World

```rust
extern crate chakracore as js;

fn main() {
  let runtime = js::Runtime::new().unwrap();
  let context = js::Context::new(&runtime).unwrap();
  let guard = context.make_current().unwrap();

  let result = js::script::eval(&guard, "(5 + 5)").unwrap();
  assert_eq!(result.to_integer(&guard), 10);
}
```

### Function - Multiply

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
