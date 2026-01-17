# Release Notes - v0.1.0

We are excited to announce the first official release of **skp-validator**, the most advanced, high-performance, and modular validation framework for the Rust ecosystem.

## 🌟 Highlights

- **Rust 2024 Ready**: Built from the ground up for the latest Rust Edition with 100% Clippy-clean code.
- **Extreme Performance**: Capable of validating millions of fields per second; optimized for high-throughput microservices.
- **Declarative & Modular**: Use powerful procedural macros for struct validation or a flexible configuration API for runtime JSON validation.
- **Extensive Rule Set**: Over 30+ built-in validation rules covering strings, numbers, collections, and temporal data.

## 🚀 Key Features

### Core Logic
- **Structured Error Reporting**: Nested, path-aware error format (e.g., `items[0].product.id`) that is natively serializable to JSON.
- **Contextual Validation**: Access runtime metadata (environment, user roles, etc.) and other field values within your validation rules.
- **Field Transformations**: Built-in support for `trim`, `uppercase`, `lowercase`, and `capitalize` operations during the validation lifecycle.

### Integrations
- **Web Adapters**: First-class support for **Axum** and **Actix-Web** via the `ValidatedJson` extractor.
- **JSON Schema**: Automatic generation of standard JSON Schemas from your Rust types using `schemars 1.2.0` integration.
- **Serde Support**: Minimal and optional Serde integration for maximum efficiency.

### Validation Rules
- **String**: Length, Email, URL, IP (v4/v6), UUID, Phone, Regex Patterns, ASCII, Alphanumeric, Contains.
- **Numeric**: Range (min/max/exclusive), MultipleOf.
- **Temporal**: Date formatting, Date Ranges, Age calculation from DOB.
- **Collection**: Unique Items, Length, and Recursive "Diving" into nested structures.
- **Comparison**: Required, MustMatch (field equality), AllowedValues (whitelisting).

## 📦 Getting Started

Add the following to your `Cargo.toml`:

```toml
[dependencies]
skp-validator = "0.1.0"
```

Then, simply derive `Validate` on your structs:

```rust
use skp_validator::Validate;

#[derive(Validate)]
struct User {
    #[validate(required, length(min = 3))]
    name: String,
    #[validate(email)]
    email: String,
}
```

## 🛠 Internal Improvements (since pre-release)
- Upgraded `schemars` to **1.2.0** with a full refactor of the schema enrichment engine to use the new value-based architecture.
- Optimized and verified the publication pipeline for multi-crate workspaces.
- Resolved all "Let Chains" and "Gen-keyword" related issues for Rust 2024 compatibility.

## 🤝 Acknowledgments
Thank you to everyone who contributed to the planning, implementation, and verification of this framework.

---
For more details, visit the [Documentation](https://docs.rs/skp-validator) or the [GitHub Repository](https://github.com/setulabs/skp-validator).
