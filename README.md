# clauser

[![crates.io badge](https://img.shields.io/crates/v/clauser)](https://crates.io/crates/clauser) [![docs.rs badge](https://img.shields.io/docsrs/clauser)](https://docs.rs/clauser/latest/clauser/)

clauser is a library for working with configuration, script, and data files from the Clausewitz engine used by Paradox Interactive for their grand strategy games.

It currently implements a Tokenizer, a low-level Reader, a serde-based Deserializer, and an copying Value deserializer for situations where serde won't work. For more information, [read the documentation](https://docs.rs/clauser/latest/clauser/).

## Examples

Using serde:
```
use serde::Deserialize;

#[derive(Deserialize)]
struct TestObject {
  a: i32,
  b: String,
  c: Date
}

let obj = clauser::de::from_str::<TestObject>(
  "a = 1 b = test c = 1940.1.1"
);
assert!(obj.a == 1);
assert!(obj.b == "test");
assert!(obj.c == Date::new(1940, 1, 1, 0));
```