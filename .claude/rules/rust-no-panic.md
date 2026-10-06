---
paths:
  - "rust/**/*.rs"
---

# Rust panic policy

## Library code (`src/`)

Allowed only: `todo!`, `unimplemented!`, `unreachable!`.

Forbidden: `panic!`, `unwrap`, `expect`, `unwrap_err`, `unwrap_unchecked`,
`assert!`, `assert_eq!`, `assert_ne!`. Recoverable failure is `Result`:
`contract::Status` in the runtime crate; in the plugin, an error that
reaches protoc as `CodeGeneratorResponse.error` and names the proto
element (file, service, method) it is about.

```rust
// BAD
let id = map.get("id").unwrap();
panic!("missing id");

// GOOD
fn load_id(map: &Map) -> Result<&str, Status> {
    match map.get("id").and_then(Value::as_str) {
        Some(id) => Ok(id),
        None => Err(Status::invalid_argument("missing id")),
    }
}

// GOOD — placeholder until the next stage implements it
todo!("functionality is not implemented")
```

## Tests (`tests/`, `#[cfg(test)]`)

`assert!`, `assert_eq!`, `assert_ne!`, `panic!`, `unwrap`, and `expect` are
allowed.
