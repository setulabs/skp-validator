# skp-validator-derive

Powerful procedural macros for `skp-validator`.

Provides the `#[derive(Validate)]` macro and associated attributes to enable declarative validation on your structs.

## Usage

```rust
use skp_validator::Validate;

#[derive(Validate)]
struct User {
    #[validate(required, length(min = 3))]
    name: String,
}
```
