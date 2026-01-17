use skp_validator::{
    Validate, ValidationContext, ValidationError, ValidationErrors, ValidationResult,
};
use skp_validator_rules::numeric::range::RangeRule;
#[cfg(feature = "regex")]
use skp_validator_rules::string::pattern::PatternRule;
use skp_validator_core::Rule; // Import core Rule trait properly
use serde::{Deserialize, Serialize};

// =============================================================================
// Domain Models
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct TaxInfo {
    country_code: String,
    tax_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransferRequest {
    amount: f64,
    currency: String,
    recipient_id: String,
    is_international: bool,
    tax_info: Option<TaxInfo>,
}

// =============================================================================
// Manual Validation Implementation
// =============================================================================

impl Validate for TaxInfo {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // 1. Validate Country Code (Simulated lookup)
        let allowed_countries = ["US", "CA", "GB", "DE"];
        if !allowed_countries.contains(&self.country_code.as_str()) {
             errors.add_field_error(
                 "country_code", 
                 ValidationError::new("country_code", "invalid_country", "Country not supported")
             );
        }

        // 2. Conditional Regex based on Country
        #[cfg(feature = "regex")]
        {
            let pattern_rule = match self.country_code.as_str() {
                "US" => Some(PatternRule::new(r"^\d{9}$").message("US Tax ID must be 9 digits (SSN/EIN)")),
                "CA" => Some(PatternRule::new(r"^[A-Z]{2}\d{6}$").message("Canadian Tax ID must be 2 letters + 6 digits")),
                _ => None,
            };

            if let Some(rule) = pattern_rule {
                if let Err(e) = rule.validate(&self.tax_id, ctx) {
                     // PatternRule creates a root error, but we want to map it to tax_id
                     // Extract the error message string for simplicity in demo
                     let msg = e.to_flat_map().values().next().and_then(|v| v.first().cloned())
                        .unwrap_or_else(|| "Invalid format".to_string());
                     
                     errors.add_field_error("tax_id", ValidationError::new("tax_id", "pattern", msg));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for TransferRequest {
    fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();

        // ---------------------------------------------------------------------
        // Scenario A: Dynamic Limits based on User Tier
        // ---------------------------------------------------------------------
        
        // 1. Get tier and limit from context
        let tier = ctx.get_meta("user_tier").unwrap_or("basic");
        // default to 1000.0 if not found
        let base_limit = match tier {
            "vip" => 100_000.0,
            "premium" => 10_000.0,
            _ => 1_000.0,
        };

        // 2. Check if specific daily limit override exists in context (e.g. from DB)
        // Metadata is string-string, so we parse it.
        let daily_limit = ctx.get_meta("daily_limit")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(base_limit);

        // 3. Apply RangeRule dynamically
        let range_rule = RangeRule::<f64>::new().max(daily_limit).message(format!(
            "Amount exceeds your daily limit of {} ({})", daily_limit, tier
        ));

        if let Err(mut e) = range_rule.validate(&self.amount, ctx) {
             let err = if !e.errors.is_empty() {
                 e.errors.pop()
             } else if let Some(field_errors) = e.fields.get_mut("") {
                 if let skp_validator::FieldErrors::Simple(vec) = field_errors {
                     vec.first().cloned()
                 } else { None }
             } else { None };

             if let Some(err_obj) = err {
                 errors.add_field_error("amount", err_obj);
             }
        }

        // ---------------------------------------------------------------------
        // Scenario B: Multi-field Dependency (International)
        // ---------------------------------------------------------------------

        // If international, tax_info is REQUIRED
        if self.is_international {
            if self.tax_info.is_none() {
                errors.add_field_error(
                    "tax_info", 
                    ValidationError::new("tax_info", "required", "Tax info required for international transfers")
                );
            }
        }

        // Validation of nested tax_info if present
        if let Some(ref info) = self.tax_info {
            if let Err(nested_errs) = info.validate_with_context(ctx) {
                errors.add_nested_errors("tax_info", nested_errs);
            }
        }
        
        // ---------------------------------------------------------------------
        // Scenario C: Contextual Feature Flag
        // ---------------------------------------------------------------------
        // If "crypto_transfers" feature is disabled in context, currency cannot be "BTC"
        
        let crypto_enabled = ctx.get_meta("feature_crypto").map(|s| s == "true").unwrap_or(false);
        
        if !crypto_enabled && self.currency == "BTC" {
             errors.add_field_error(
                 "currency",
                 ValidationError::new("currency", "feature_disabled", "Crypto transfers are disabled")
             );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =============================================================================
// Main Execution
// =============================================================================

fn main() {
    println!("--- Test Case 1: Basic User exceeding limit ---");
    let req1 = TransferRequest {
        amount: 5000.0,
        currency: "USD".to_string(),
        recipient_id: "user_123".to_string(),
        is_international: false,
        tax_info: None,
    };
    let ctx_basic = ValidationContext::new().with_meta("user_tier", "basic");
    print_validation("Basic User", &req1, &ctx_basic);


    println!("\n--- Test Case 2: VIP User within limit ---");
    let req2 = TransferRequest {
        amount: 50000.0,
        currency: "USD".to_string(),
        recipient_id: "user_123".to_string(),
        is_international: false,
        tax_info: None,
    };
    let ctx_vip = ValidationContext::new().with_meta("user_tier", "vip");
    print_validation("VIP User", &req2, &ctx_vip);


    println!("\n--- Test Case 3: International Transfer Missing Tax Info ---");
    let req3 = TransferRequest {
        amount: 100.0,
        currency: "USD".to_string(),
        recipient_id: "user_999".to_string(),
        is_international: true, // Requires tax_info
        tax_info: None,
    };
    print_validation("Intl Missing Tax", &req3, &ctx_basic);


    println!("\n--- Test Case 4: International Transfer Invalid Tax Format (US) ---");
    let req4 = TransferRequest {
        amount: 100.0,
        currency: "USD".to_string(),
        recipient_id: "user_999".to_string(),
        is_international: true,
        tax_info: Some(TaxInfo {
            country_code: "US".to_string(),
            tax_id: "AB123".to_string(), // Invalid for US, valid for maybe others?
        }),
    };
    print_validation("Intl Invalid US Tax", &req4, &ctx_basic);


    println!("\n--- Test Case 5: Feature Flag (Crypto Blocked) ---");
    let req5 = TransferRequest {
        amount: 10.0,
        currency: "BTC".to_string(),
        recipient_id: "wallet_123".to_string(),
        is_international: false,
        tax_info: None,
    };
    print_validation("Crypto Blocked", &req5, &ctx_basic); // Feature flag missing = false


    println!("\n--- Test Case 6: Feature Flag (Crypto Allowed) ---");
    let ctx_crypto = ValidationContext::new().with_meta("feature_crypto", "true");
    print_validation("Crypto Allowed", &req5, &ctx_crypto);
}

fn print_validation(name: &str, req: &TransferRequest, ctx: &ValidationContext) {
    match req.validate_with_context(ctx) {
        Ok(_) => println!("✅ {}: Valid", name),
        Err(e) => {
            println!("❌ {}: Invalid", name);
            // Print flat errors for clarity
            for (field, msgs) in e.to_flat_map() {
                for msg in msgs {
                    println!("   - {}: {}", field, msg);
                }
            }
        }
    }
}
