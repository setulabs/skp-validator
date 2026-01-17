# skp-validator-actix

High-performance Actix Web integration for `skp-validator`.

## Usage

Supports the `ValidatedJson` extractor for automatic payload validation.

```rust
#[post("/register")]
async fn register_user(user: ValidatedJson<RegisterUser>) -> impl Responder {
    format!("Welcome {}!", user.name)
}
```
