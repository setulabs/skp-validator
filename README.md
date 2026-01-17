# SKP Validator

A comprehensive, high-performance, and reusable validation framework for Rust structs and JSON objects, fully compatible with **Rust 2024 Edition**. Provides declarative validation rules without hardcoding or service-specific logic, ensuring type safety and extreme throughput.

## 🚀 Features

- **Declarative Validation**: Use derive macros and attributes on your structs.
- **Modern Rust**: Fully optimized for **Rust 2024 Edition** with 100% Clippy clean code.
- **Comprehensive Rules**: Includes length, email, url, ip, uuid, phone, pattern, ascii, alphanumeric, numeric ranges, dates, age, and more.
- **Contextual & Dependency Validation**: Rules that depend on other field values or runtime context.
- **Collection Validation**: Support for `unique_items` and diving into `Vec`, `HashMap`, and other collections.
- **Field Transformations**: Uppercase, lowercase, and trim operations during validation.
- **Framework Integration**: Built-in adapters for **Axum** and **Actix-Web**.
- **JSON Schema Support**: Generate JSON Schemas from your validation rules.
- **Extreme Performance**: Optimized for high-throughput scenarios (capable of validating 10,000+ items in microseconds).

## ✨ Feature Matrix

| Feature | Use Case | Pros |
|---------|----------|------|
| **Declarative Validation** | Primary way to validate structs using `#[derive(Validate)]` and attributes. | Type-safe, concise, and integrates with Rust's type system. |
| **Runtime JSON Validation** | Validating raw JSON objects without needing to deserialize into a struct first. | Extremely high performance, avoids double processing, ideal for proxy layers and dynamic schemas. |
| **Field Dependencies** | Conditional rules where one field's validation depends on another field's value. | Handles complex business logic (e.g., "required if X is delivery") natively within the validation layer. |
| **Field Transformations** | Automatically trimming, uppercasing, or lowercasing field values during the validation process. | Ensures data consistency and sanitization without boilerplate code. |
| **Nested & Collection Diving** | Deeply validating nested structs or iterating through collections like `Vec` or `HashMap`. | Full path tracking for complex data structures, ensuring every leaf node is validated. |
| **Framework Adapters** | Direct integration with Axum and Actix-Web via extractor types. | Seamless developer experience in web services; returns standardized validation error responses automatically. |
| **Contextual Validation** | Accessing the full validation context (all fields, metadata) within a rule closure. | Maximum flexibility for complex cross-field checks that require more than simple equality. |
| **Recursive Schemas** | Automatic validation of recursive data structures (e.g., Tree nodes). | Essential for modern, complex data models with infinite depth. |

## 📦 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
skp-validator = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

Basic usage with derive macros:

```rust
use skp_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct User {
    #[validate(required, length(min = 3, max = 50))]
    pub name: String,

    #[validate(required, email)]
    pub email: String,

    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,

    #[validate(required, alphanumeric)]
    pub username: String,
}

// Validate a struct
let user = User {
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    age: Some(25),
    username: "johndoe123".to_string(),
};

match user.validate() {
    Ok(_) => println!("User is valid!"),
    Err(errors) => {
        println!("Validation failed: {}", errors);
    }
}
```

## 📦 Core Modules

- **`skp-validator-core`**: The engine and base traits (`Validate`, `Rule`).
- **`skp-validator-rules`**: A rich set of built-in validation rules and transforms.
- **`skp-validator-derive`**: Powerful proc-macros for declarative validation.
- **`skp-validator-jsonschema`**: Integration with `schemars` for schema generation.
- **`skp-validator-axum`** & **`skp-validator-actix`**: First-class web framework extractors.

## 🏃 Usage Examples

### Basic Struct Validation

```rust
use skp_validator::Validate;

#[derive(Validate)]
struct Product {
    #[validate(required, length(min = 2, max = 100))]
    pub name: String,

    #[validate(required, range(min = 0.01, max = 100000.0))]
    pub price: f64,

    #[validate(allowed_values = ["active", "inactive", "discontinued"])]
    pub status: String,
}

let product = Product {
    name: "Laptop".to_string(),
    price: 999.99,
    status: "active".to_string(),
};

assert!(product.validate().is_ok());
```

### Date and Age Validation

```rust
use skp_validator::Validate;

#[derive(Validate)]
struct Person {
    #[validate(required, date(format = "%Y-%m-%d"))]
    pub birth_date: String,

    #[validate(age(min = 18, max = 100, date_format = "%Y-%m-%d"))]
    pub age: Option<u32>,
}
```

### Collection Diving

```rust
use skp_validator::Validate;

#[derive(Validate)]
struct Order {
    #[validate(required, length(min = 1), dive)]
    pub items: Vec<OrderItem>,
    
    #[validate(unique_items)]
    pub tags: Vec<String>,
}

#[derive(Validate)]
struct OrderItem {
    #[validate(required)]
    pub product_id: String,
    #[validate(range(min = 1))]
    pub quantity: u32,
}
```

### Dependency Validation

Validate a field based on the value of another field using the `DependencyRule`:

```rust
use skp_validator_rules::custom::dependency::{DependencyRule, DependencyCondition};
use skp_validator_core::{Rule, ValidationContext};

// Shipping address is required only when shipping_method is "delivery"
let rule = DependencyRule::<str>::new("shipping_address")
    .depends_on("shipping_method", DependencyCondition::Equals("delivery".to_string()))
    .then_required();
```

### JSON Validation

```rust
use skp_validator::*;
use serde_json::json;

// Create validation configuration
let mut config = ValidationConfig::new();

let name_config = FieldValidationConfig::new("name")
    .push_rule(ValidationRule::Required { message: None })
    .push_rule(ValidationRule::Length { min: Some(2), max: Some(50), message: None });

config = config.add_field(name_config);

// Validate JSON object
let json_data = json!({
    "name": "John Doe",
    "age": 25,
});

let mut context = ValidationContext::from_json(&json_data);
match config.validate(&mut context) {
    Ok(_) => println!("JSON is valid!"),
    Err(errors) => {
        println!("Validation failed: {}", errors);
    }
}
```

### Custom Validators

```rust
use skp_validator::*;

#[derive(Validate)]
struct PasswordRequest {
    #[validate(required, length(min = 8))]
    pub password: String,

    #[validate(required, must_match = "password")]
    pub confirm_password: String,
}
```

### JSON Schema Generation

Generate standard JSON Schemas from your Rust types:

```rust
use skp_validator_jsonschema::schema_for;
use skp_validator::Validate;
use schemars::JsonSchema;

#[derive(JsonSchema, Validate)]
struct Signup {
    #[validate(email)]
    pub email: String,
    #[validate(range(min = 18))]
    pub age: u32,
}

let schema = schema_for::<Signup>();
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```

### Web Framework Integration

Validate incoming JSON payloads seamlessly in popular web frameworks:

#### Axum
```rust
use axum::{routing::post, Router};
use skp_validator_axum::ValidatedJson;
use skp_validator::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(required, length(min = 3))]
    name: String,
    #[validate(email)]
    email: String,
}

async fn create_user(ValidatedJson(user): ValidatedJson<CreateUser>) -> String {
    format!("Created user: {}", user.name)
}

let app = Router::new().route("/users", post(create_user));
```

#### Actix-Web
```rust
use actix_web::{post, Responder};
use skp_validator_actix::ValidatedJson;
use skp_validator::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct RegisterUser {
    #[validate(length(min = 3))]
    name: String,
}

#[post("/register")]
async fn register_user(user: ValidatedJson<RegisterUser>) -> impl Responder {
    format!("Welcome {}!", user.name)
}
```

## 🔧 Advanced Usage

### Manual Configuration

```rust
use skp_validator::*;

// Create validation configuration manually
let mut config = ValidationConfig::new();

// Add field validations
let email_config = FieldValidationConfig::new("email")
    .push_rule(ValidationRule::Required { message: None })
    .push_rule(ValidationRule::Email { message: None });

config = config.add_field(email_config);
```

## 📋 Validation Rules Reference

| Rule | Description | Example |
|------|-------------|---------|
| `required` | Field must not be null/empty | `#[validate(required)]` |
| `length(min, max, equal)` | String or Collection length | `#[validate(length(min = 3, max = 50))]` |
| `email` | Standard email format | `#[validate(email)]` |
| `url` | URL format validation | `#[validate(url)]` |
| `ip` | IP address (v4 or v6) | `#[validate(ip)]` |
| `uuid` | UUID format | `#[validate(uuid)]` |
| `phone` | International phone format | `#[validate(phone)]` |
| `ascii` | ASCII characters only | `#[validate(ascii(printable = true))]` |
| `alphanumeric` | Alphanumeric characters only | `#[validate(alphanumeric)]` |
| `range(min, max)` | Numeric range check | `#[validate(range(min = 18, max = 120))]` |
| `multiple_of(n)` | Numeric multiplicity check | `#[validate(multiple_of = 5)]` |
| `pattern(regex)` | Regular expression match | `#[validate(pattern = r"^\d{10}$")]` |
| `unique_items` | Enforce unique items in collections | `#[validate(unique_items)]` |
| `date(format)` | Valid date string match | `#[validate(date(format = "%Y-%m-%d"))]` |
| `age(min, max)` | Age calculation from DOB | `#[validate(age(min = 18))]` |
| `must_match(field)` | Field value must match another field | `#[validate(must_match = "confirm_password")]` |
| `allowed_values(v)` | Whitelist of allowed values | `#[validate(allowed_values = ["A", "B"])]` |
| **Transforms** | **Description** | **Example** |
| `uppercase` | Convert value to uppercase | `#[validate(uppercase)]` |
| `lowercase` | Convert value to lowercase | `#[validate(lowercase)]` |
| `trim` | Trim whitespace | `#[validate(trim)]` |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
