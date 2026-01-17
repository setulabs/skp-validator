# OneClick Validator

A comprehensive, reusable validation framework for Rust structs and JSON objects. Provides declarative validation rules without hardcoding or service-specific logic, ensuring type safety and high performance.

## 🚀 Features

- **Declarative Validation**: Use derive macros and attributes on your structs
- **Multiple Validation Types**: Required, length, regex, numeric ranges, dates, custom validators
- **Contextual Validation**: Rules that depend on other field values
- **Field Transformations**: Uppercase, lowercase, and trim operations during validation
- **Dependency Validation**: Conditional validation based on other field values
- **JSON Support**: Validate JSON objects directly without deserialization
- **Custom Error Messages**: Detailed validation error reporting
- **Zero External Dependencies**: Pure validation logic, no framework coupling
- **Type Safety**: Compile-time guarantees with Rust's type system
- **Thread Safety**: Arc-based closures ensure thread-safe validation

## 📦 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
oc_validator = "0.0.1"
serde = { version = "1.0", features = ["derive"] }
```

Basic usage with derive macros:

```rust
use oc_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct User {
    #[validate(required, length(min = 3, max = 50))]
    pub name: String,

    #[validate(required, email)]
    pub email: String,

    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,

    #[validate(required, regex(pattern = r"^\+?[1-9]\d{1,14}$"))]
    pub phone: Option<String>,
}

// Validate a struct
let user = User {
    name: "John".to_string(),
    email: "john@example.com".to_string(),
    age: Some(25),
    phone: Some("+1234567890".to_string()),
};

match user.validate() {
    Ok(_) => println!("User is valid!"),
    Err(errors) => {
        for error in errors {
            println!("Validation error: {}", error);
        }
    }
}
```

## 📦 Modules

### `rules`

Core validation rule system:

- `ValidationRule` - Comprehensive enum of all validation types
- `FieldValidationConfig` - Configuration for individual fields
- `ValidationConfig` - Complete validation configuration
- `FieldDependency` - Conditional validation based on other fields

### `context`

Validation execution context:

- `ValidationContext` - Runtime context with field values and metadata
- Field value access methods (string, number, boolean, etc.)
- Metadata storage for custom validation data

### `validator`

Main validation traits and functions:

- `Validate` - Core trait for validatable structs
- `validate_object()` - Helper for validating any serializable object
- `validate_json_str()` - Validate JSON strings directly
- `validate_json_value()` - Validate serde_json::Value objects

### `error`

Comprehensive error handling:

- `FieldError` - Individual field validation errors
- `ValidationResult<T>` - Result type for validation operations
- Detailed error messages with field names and custom messages

### `json_validator`

JSON-specific validation:

- Direct JSON object validation without deserialization
- Path-based field access for nested JSON structures
- Type-aware validation for JSON values

## 🏃 Usage Examples

### Basic Struct Validation

```rust
use oc_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Product {
    #[validate(required, length(min = 2, max = 100))]
    pub name: String,

    #[validate(required, range(min = 0.01, max = 100000.0))]
    pub price: f64,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    #[validate(allowed_values = ["active", "inactive", "discontinued"])]
    pub status: String,
}

let product = Product {
    name: "Laptop".to_string(),
    price: 999.99,
    description: Some("High-performance laptop".to_string()),
    status: "active".to_string(),
};

assert!(product.validate().is_ok());
```

### Date and Age Validation

```rust
use oc_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Person {
    #[validate(required)]
    pub name: String,

    #[validate(required, date(format = "%Y-%m-%d"))]
    pub birth_date: String,

    #[validate(age(min = 18, max = 100, date_format = "%Y-%m-%d"))]
    pub age: Option<u32>,
}

let person = Person {
    name: "Alice".to_string(),
    birth_date: "1990-05-15".to_string(),
    age: Some(34),
};

assert!(person.validate().is_ok());
```

### Field Transformations

```rust
use oc_validator::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Validate)]
struct UserProfile {
    #[validate(required, length(min = 3, max = 20), uppercase)]
    pub username: String,

    #[validate(required, email, lowercase)]
    pub email: String,

    #[validate(trim)]
    pub bio: Option<String>,
}

let profile = UserProfile {
    username: "john_doe".to_string(),  // Will be transformed to "JOHN_DOE"
    email: "John.Doe@Example.COM".to_string(),  // Will be transformed to "john.doe@example.com"
    bio: Some("  Hello World  ".to_string()),  // Will be transformed to "Hello World"
};

let result = profile.validate();
assert!(result.is_ok());

// Access transformed values
if let Ok(_) = result {
    println!("Username: {}", profile.username);  // "JOHN_DOE"
    println!("Email: {}", profile.email);        // "john.doe@example.com"
    println!("Bio: {:?}", profile.bio);         // Some("Hello World")
}
```

### Dependency Validation

```rust
use oc_validator::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct AccountOpeningForm {
    pub account_type: String,
    pub employment_type: Option<String>,
    pub employer_name: Option<String>,
    pub minimum_balance: Option<f64>,
}

impl AccountOpeningForm {
    pub fn validation_config() -> ValidationConfig {
        let mut config = ValidationConfig::new();

        // Account type validation
        let account_config = FieldValidationConfig::new("account_type")
            .add_rule(ValidationRule::Required { message: None })
            .add_rule(ValidationRule::AllowedValues {
                values: vec!["SAVINGS".to_string(), "CURRENT".to_string(), "TRADING".to_string()],
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);

        config = config.add_field(account_config);

        // Employment type validation
        let employment_config = FieldValidationConfig::new("employment_type")
            .add_rule(ValidationRule::AllowedValues {
                values: vec!["SALARIED".to_string(), "SELF_EMPLOYED".to_string()],
                message: None,
            })
            .add_rule(ValidationRule::Uppercase);

        config = config.add_field(employment_config);

        // Employer name dependency - required only for salaried employees
        let employer_dependency = FieldDependency::new(
            "employer_name",
            "employment_type",
            Arc::new(|ctx| {
                ctx.get_string("employment_type")
                    .map(|et| et == "SALARIED")
                    .unwrap_or(false)
            }),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Employer name is required for salaried employees".to_string()),
        })
        .add_rule(ValidationRule::Length {
            min: Some(2),
            max: Some(100),
            message: None,
        });

        config = config.add_dependency(employer_dependency);

        // Minimum balance dependency - different requirements per account type
        let balance_dependency = FieldDependency::new(
            "minimum_balance",
            "account_type",
            Arc::new(|ctx| ctx.has_value("account_type")),
        )
        .add_rule(ValidationRule::Required {
            message: Some("Minimum balance is required".to_string()),
        })
        .add_rule(ValidationRule::Contextual {
            validator: Arc::new(|ctx| {
                let account_type = ctx.get_string("account_type").unwrap_or("");
                let balance = ctx.get_number("minimum_balance").unwrap_or(0.0);

                let min_required = match account_type {
                    "SAVINGS" => 1000.0,
                    "CURRENT" => 5000.0,
                    "TRADING" => 25000.0,
                    _ => 0.0,
                };

                if balance < min_required {
                    return Err(vec![FieldError::new(
                        "minimum_balance",
                        format!("{} account requires minimum ₹{:.0}", account_type, min_required),
                    )]);
                }

                Ok(())
            }),
            message: None,
        });

        config = config.add_dependency(balance_dependency);

        config
    }

    pub fn validate_advanced(&self) -> ValidationResult<()> {
        let config = Self::validation_config();
        let mut context = ValidationContext::from_serde(self)?;
        config.validate(&mut context)
    }
}

// Valid salaried employee with savings account
let valid_form = AccountOpeningForm {
    account_type: "savings".to_string(),  // Will be transformed to "SAVINGS"
    employment_type: Some("salaried".to_string()),  // Will be transformed to "SALARIED"
    employer_name: Some("Tech Corp".to_string()),
    minimum_balance: Some(5000.0),  // Meets savings account requirement
};

assert!(valid_form.validate_advanced().is_ok());

// Invalid - salaried employee without employer name
let invalid_form = AccountOpeningForm {
    account_type: "savings".to_string(),
    employment_type: Some("salaried".to_string()),
    employer_name: None,  // Missing required field
    minimum_balance: Some(5000.0),
};

assert!(invalid_form.validate_advanced().is_err());
```

### JSON Validation

```rust
use oc_validator::*;
use serde_json::json;

// Create validation configuration
let mut config = ValidationConfig::new();

let name_config = FieldValidationConfig::new("name")
    .add_rule(ValidationRule::Required { message: None })
    .add_rule(ValidationRule::Length { min: Some(2), max: Some(50), message: None });

let age_config = FieldValidationConfig::new("age")
    .add_rule(ValidationRule::Range { min: Some(18.0), max: Some(120.0), message: None });

config = config.add_field(name_config);
config = config.add_field(age_config);

// Validate JSON object
let json_data = json!({
    "name": "John Doe",
    "age": 25,
    "email": "john@example.com"
});

let mut context = ValidationContext::from_json(&json_data);
match config.validate(&mut context) {
    Ok(_) => println!("JSON is valid!"),
    Err(errors) => {
        for error in errors {
            println!("Validation error: {}", error);
        }
    }
}
```

### Custom Validators

```rust
use oc_validator::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct PasswordChangeRequest {
    #[validate(required, length(min = 8, max = 128))]
    pub current_password: String,

    #[validate(required, length(min = 8, max = 128))]
    pub new_password: String,

    #[validate(required, length(min = 8, max = 128))]
    pub confirm_password: String,

    #[validate(custom = "validate_password_confirmation")]
    pub _password_confirmation: Option<String>,  // Dummy field for custom validation
}

fn validate_password_confirmation(ctx: &ValidationContext) -> ValidationResult<()> {
    let new_password = ctx.get_string("new_password").unwrap_or("");
    let confirm_password = ctx.get_string("confirm_password").unwrap_or("");

    if new_password != confirm_password {
        return Err(vec![FieldError::new(
            "confirm_password",
            "Password confirmation does not match".to_string(),
        )]);
    }

    // Additional password strength checks
    if new_password.len() < 8 {
        return Err(vec![FieldError::new(
            "new_password",
            "Password must be at least 8 characters long".to_string(),
        )]);
    }

    if !new_password.chars().any(|c| c.is_uppercase()) {
        return Err(vec![FieldError::new(
            "new_password",
            "Password must contain at least one uppercase letter".to_string(),
        )]);
    }

    if !new_password.chars().any(|c| c.is_lowercase()) {
        return Err(vec![FieldError::new(
            "new_password",
            "Password must contain at least one lowercase letter".to_string(),
        )]);
    }

    if !new_password.chars().any(|c| c.is_numeric()) {
        return Err(vec![FieldError::new(
            "new_password",
            "Password must contain at least one number".to_string(),
        )]);
    }

    Ok(())
}

let request = PasswordChangeRequest {
    current_password: "oldpass123".to_string(),
    new_password: "NewPass123".to_string(),
    confirm_password: "NewPass123".to_string(),
    _password_confirmation: None,
};

assert!(request.validate().is_ok());
```

## 🔧 Advanced Usage

### Manual Configuration

```rust
use oc_validator::*;

// Create validation configuration manually
let mut config = ValidationConfig::new();

// Add field validations
let email_config = FieldValidationConfig::new("email")
    .add_rule(ValidationRule::Required {
        message: Some("Email is required".to_string()),
    })
    .add_rule(ValidationRule::Email {
        message: Some("Invalid email format".to_string()),
    });

config = config.add_field(email_config);

// Add dependencies
let credit_card_dependency = FieldDependency::new(
    "credit_card",
    "vip_status",
    Arc::new(|ctx| ctx.get_bool("vip_status").unwrap_or(false)),
)
.add_rule(ValidationRule::Required {
    message: Some("Credit card required for VIP members".to_string()),
});

config = config.add_dependency(credit_card_dependency);

// Validate
let json_data = serde_json::json!({
    "email": "user@example.com",
    "vip_status": true,
    "credit_card": "1234567890123456"
});

let mut context = ValidationContext::from_json(&json_data);
assert!(config.validate(&mut context).is_ok());
```

### Error Handling

```rust
use oc_validator::*;

#[derive(Debug, serde::Deserialize, Validate)]
struct RegistrationForm {
    #[validate(required, email)]
    pub email: String,

    #[validate(required, length(min = 8))]
    pub password: String,

    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,
}

let form = RegistrationForm {
    email: "invalid-email",  // Invalid email
    password: "short",       // Too short
    age: Some(150),          // Too old
};

match form.validate() {
    Ok(_) => println!("Form is valid"),
    Err(errors) => {
        println!("Validation failed with {} errors:", errors.len());
        for error in errors {
            println!("  {}: {}", error.field, error.message);
        }
    }
}
```

## 📋 Validation Rules Reference

| Rule | Description | Example |
|------|-------------|---------|
| `required` | Field must not be null/empty | `#[validate(required)]` |
| `length(min = 5, max = 100)` | String length constraints | `#[validate(length(min = 3, max = 50))]` |
| `regex(pattern = "...")` | Regular expression match | `#[validate(regex(pattern = r"^\d{10}$"))]` |
| `email` | Email format validation | `#[validate(email)]` |
| `phone` | Phone number format | `#[validate(phone)]` |
| `range(min = 0, max = 100)` | Numeric range | `#[validate(range(min = 18, max = 120))]` |
| `int_range(min = 1, max = 10)` | Integer range | `#[validate(int_range(min = 1, max = 5))]` |
| `date(format = "%Y-%m-%d")` | Date format validation | `#[validate(date(format = "%d-%m-%Y"))]` |
| `age(min = 18, max = 100, date_format = "%Y-%m-%d")` | Age validation from DOB | `#[validate(age(min = 21, date_format = "%Y-%m-%d"))]` |
| `allowed_values = ["A", "B", "C"]` | Whitelist values | `#[validate(allowed_values = ["active", "inactive"])]` |
| `uppercase` | Transform to uppercase | `#[validate(uppercase)]` |
| `lowercase` | Transform to lowercase | `#[validate(lowercase)]` |
| `trim` | Remove whitespace | `#[validate(trim)]` |
| `custom = "function_name"` | Custom validation function | `#[validate(custom = "validate_password")]` |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
