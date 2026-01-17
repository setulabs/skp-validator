# skp-validator-axum

Seamless integration of `skp-validator` with the Axum web framework.

## Usage

Provides a `ValidatedJson` extractor that automatically validates incoming JSON payloads and returns standard validation error responses.

```rust
async fn create_user(ValidatedJson(user): ValidatedJson<CreateUser>) -> String {
    format!("Created user: {}", user.name)
}
```
