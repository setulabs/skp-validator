use skp_validator::{
    Validate, ValidationContext, ValidationErrors, ValidationResult,
};
use skp_validator_rules::custom::{
    contextual::ContextualRule,
    dependency::{DependencyCondition, DependencyRule},
};
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Scenario 1: Simple Dependency using Derive Macro
// The `must_match` rule is supported directly by the derive macro.
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
struct PasswordChange {
    #[validate(length(min = 8))]
    new_password: String,

    #[validate(must_match(other = "new_password", message = "Passwords do not match"))]
    confirm_password: String,
}

// -----------------------------------------------------------------------------
// Scenario 2: Advanced Dependency (Manual Implementation)
// Rules:
// - If `shipping_method` is "delivery", `address` is required.
// - If `shipping_method` is "pickup", `address` must be empty (or ignored).
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    shipping_method: String,
    address: Option<String>,
}

impl Validate for Order {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // We need to add fields to context manually for DependencyRule to work
        // because DependencyRule expects to read dependent values from the context.
        // In a real framework adapter, this population happens automatically.
        let local_ctx = ctx.clone().with_field("shipping_method", &self.shipping_method);
        
        let rule = DependencyRule::<String>::new("address")
            .depends_on("shipping_method", DependencyCondition::Equals("delivery".to_string()))
            .then_required()
            .message("Address is required for delivery");

        // Convert Option to &String or empty string for validation
        let val_to_check = self.address.as_deref().unwrap_or("").to_string();
        
        if let Err(mut e) = skp_validator_core::Rule::validate(&rule, &val_to_check, &local_ctx) {
             // Extract error from wherever it may be (root or field)
             let err = if !e.errors.is_empty() {
                 e.errors.pop()
             } else if let Some(field_errors) = e.fields.get_mut("") {
                 if let skp_validator::FieldErrors::Simple(vec) = field_errors {
                     vec.first().cloned()
                 } else { None }
             } else { None };

             if let Some(mut err_obj) = err {
                  err_obj.message = "Address is required for delivery".to_string(); // Ensure message
                  errors.add_field_error("address", err_obj);
             }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// -----------------------------------------------------------------------------
// Scenario 3: Context-Aware Validation (Manual Implementation)
// Rules:
// - If context environment is "production", feature name cannot start with "beta_".
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct FeatureFlag {
    name: String,
    enabled: bool,
}

impl Validate for FeatureFlag {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // Contextual Rule: Block beta features in production
        let rule = ContextualRule::<str>::new("env_check")
            .when(|ctx| ctx.get_meta("env") == Some("production"))
            .validate_with(|val, _| !val.starts_with("beta_"))
            .message("Beta features are not allowed in production");

        if let Err(mut e) = skp_validator_core::Rule::validate(&rule, self.name.as_str(), ctx) {
             let err = if !e.errors.is_empty() {
                 e.errors.pop()
             } else if let Some(field_errors) = e.fields.get_mut("") {
                 if let skp_validator::FieldErrors::Simple(vec) = field_errors {
                     vec.first().cloned()
                 } else { None }
             } else { None };

             if let Some(err_obj) = err {
                 errors.add_field_error("name", err_obj);
             }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn main() {
    println!("--- Scenario 1: Simple Dependency (Derive) ---");
    let invalid_pass = PasswordChange {
        new_password: "secure_password".to_string(),
        confirm_password: "wrong_password".to_string(),
    };
    match invalid_pass.validate() {
        Ok(_) => println!("Password matched!"),
        Err(e) => println!("Validation error: {}", e),
    }

    println!("\n--- Scenario 2: Advanced Dependency (Manual) ---");
    let invalid_order = Order {
        shipping_method: "delivery".to_string(),
        address: None, // Required for delivery!
    };
    // Note: We don't pass context here because the manual impl creates its own local context
    // for internal dependency resolution.
    match invalid_order.validate() {
        Ok(_) => println!("Order valid!"),
        Err(e) => println!("Order invalid: {}", e),
    }
    
    let valid_order = Order {
        shipping_method: "delivery".to_string(),
        address: Some("123 Main St".to_string()),
    };
    match valid_order.validate() {
         Ok(_) => println!("Order valid!"),
         Err(e) => println!("Order invalid: {}", e),
    }

    println!("\n--- Scenario 3: Context-Aware Validation ---");
    let feature = FeatureFlag {
        name: "beta_new_ui".to_string(),
        enabled: true,
    };

    // Case A: Dev Environment (Should Pass)
    let dev_ctx = ValidationContext::new().with_meta("env", "dev");
    match feature.validate_with_context(&dev_ctx) {
        Ok(_) => println!("Dev: Feature allowed"),
        Err(e) => println!("Dev: {}", e),
    }

    // Case B: Production Environment (Should Fail)
    let prod_ctx = ValidationContext::new().with_meta("env", "production");
    match feature.validate_with_context(&prod_ctx) {
        Ok(_) => println!("Prod: Feature allowed"),
        Err(e) => println!("Prod: Blocked - {}", e),
    }
}
