use crate::context::ValidationContext;
use crate::error::JsonValidationError;
use crate::rules::{ValidationConfig, ValidationRule};
use serde_json::Value;
use std::collections::HashMap;

/// Validate a JSON object against a validation configuration
pub fn validate_json(
    json: &str,
    config: &ValidationConfig,
) -> Result<(), Vec<JsonValidationError>> {
    let json_value: Value = serde_json::from_str(json).map_err(|e| {
        vec![JsonValidationError::new(
            "root",
            format!("JSON parsing error: {}", e),
        )]
    })?;

    validate_json_value(&json_value, config)
}

/// Validate a JSON Value against a validation configuration
pub fn validate_json_value(
    json_value: &Value,
    config: &ValidationConfig,
) -> Result<(), Vec<JsonValidationError>> {
    let context = ValidationContext::from_json(json_value);
    let mut all_errors = Vec::new();

    for (field_name, field_config) in &config.fields {
        let value = context.get_field_value(field_name);

        for rule in &field_config.rules {
            if let Err(errors) = validate_rule_with_json_error(field_name, value, rule, &context) {
                all_errors.extend(errors);
            }
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

/// Validate a single rule and convert errors to JsonValidationError
fn validate_rule_with_json_error(
    field_name: &str,
    value: Option<&Value>,
    rule: &ValidationRule,
    context: &ValidationContext,
) -> Result<(), Vec<JsonValidationError>> {
    match rule {
        ValidationRule::Required { message } => {
            if value.is_none()
                || value.unwrap().is_null()
                || (value.unwrap().is_string()
                    && value.unwrap().as_str().unwrap().trim().is_empty())
            {
                let msg = message
                    .clone()
                    .unwrap_or_else(|| format!("Field '{}' is required", field_name));
                return Err(vec![JsonValidationError::new(field_name, msg)]);
            }
        }

        ValidationRule::Length { min, max, message } => {
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    let len = s.len();
                    if let Some(min_len) = min {
                        if len < *min_len {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Field '{}' must be at least {} characters",
                                    field_name, min_len
                                )
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                    if let Some(max_len) = max {
                        if len > *max_len {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Field '{}' must be at most {} characters",
                                    field_name, max_len
                                )
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                }
            }
        }

        ValidationRule::Regex { pattern, message } => {
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if !regex.is_match(s) {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' does not match required pattern", field_name)
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                }
            }
        }

        ValidationRule::Email { message } => {
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    let email_regex = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
                    if !email_regex.is_match(s) {
                        let msg = message.clone().unwrap_or_else(|| {
                            format!("Field '{}' must be a valid email address", field_name)
                        });
                        return Err(vec![JsonValidationError::new(field_name, msg)]);
                    }
                }
            }
        }

        ValidationRule::Phone { message } => {
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    let phone_regex = regex::Regex::new(r"^\+?[\d\s\-\(\)]+$").unwrap();
                    if !phone_regex.is_match(s) {
                        let msg = message.clone().unwrap_or_else(|| {
                            format!("Field '{}' must be a valid phone number", field_name)
                        });
                        return Err(vec![JsonValidationError::new(field_name, msg)]);
                    }
                }
            }
        }

        ValidationRule::Range { min, max, message } => {
            if let Some(val) = value {
                if let Some(num) = val.as_f64() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be at least {}", field_name, min_val)
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be at most {}", field_name, max_val)
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                }
            }
        }

        ValidationRule::IntRange { min, max, message } => {
            if let Some(val) = value {
                if let Some(num) = val.as_i64() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be at least {}", field_name, min_val)
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be at most {}", field_name, max_val)
                            });
                            return Err(vec![JsonValidationError::new(field_name, msg)]);
                        }
                    }
                }
            }
        }

        ValidationRule::AllowedValues { values, message } => {
            if let Some(val) = value {
                let str_val = val.to_string().trim_matches('"').to_string();
                if !values.contains(&str_val) {
                    let msg = message.clone().unwrap_or_else(|| {
                        format!("Field '{}' must be one of: {:?}", field_name, values)
                    });
                    return Err(vec![JsonValidationError::new(field_name, msg)]);
                }
            }
        }

        ValidationRule::Custom { validator, message } => {
            if let Err(errors) = validator(context) {
                if !errors.is_empty() {
                    let msg = message
                        .clone()
                        .unwrap_or_else(|| "Custom validation failed".to_string());
                    return Err(vec![JsonValidationError::new(field_name, msg)]);
                }
            }
        }

        ValidationRule::Contextual { validator, message } => {
            if let Err(errors) = validator(context) {
                if !errors.is_empty() {
                    let msg = message
                        .clone()
                        .unwrap_or_else(|| "Contextual validation failed".to_string());
                    return Err(vec![JsonValidationError::new(field_name, msg)]);
                }
            }
        }

        // New validation rules - basic implementation for now
        ValidationRule::Date { format, message } => {
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    if chrono::NaiveDate::parse_from_str(s, format).is_err() {
                        let msg = message.clone().unwrap_or_else(|| {
                            format!(
                                "Field '{}' must be a valid date in format {}",
                                field_name, format
                            )
                        });
                        return Err(vec![JsonValidationError::new(field_name, msg)]);
                    }
                }
            }
        }

        ValidationRule::DateRange {
            min: _,
            max: _,
            format,
            message,
        } => {
            // Basic implementation - full date range validation would be more complex
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    if chrono::NaiveDate::parse_from_str(s, format).is_err() {
                        let msg = message.clone().unwrap_or_else(|| {
                            format!(
                                "Field '{}' must be a valid date in format {}",
                                field_name, format
                            )
                        });
                        return Err(vec![JsonValidationError::new(field_name, msg)]);
                    }
                }
            }
        }

        ValidationRule::Age {
            min: _,
            max: _,
            date_format,
            message,
        } => {
            // Basic implementation - full age validation would calculate age from DOB
            if let Some(val) = value {
                if let Some(s) = val.as_str() {
                    if chrono::NaiveDate::parse_from_str(s, date_format).is_err() {
                        let msg = message.clone().unwrap_or_else(|| {
                            format!(
                                "Field '{}' must be a valid date in format {}",
                                field_name, date_format
                            )
                        });
                        return Err(vec![JsonValidationError::new(field_name, msg)]);
                    }
                }
            }
        }

        ValidationRule::Uppercase | ValidationRule::Lowercase | ValidationRule::Trim => {
            // Transformation rules don't produce validation errors in JSON validation
            // They would be applied during struct validation
        }
    }

    Ok(())
}

/// Validate JSON with field-specific error paths
pub fn validate_json_with_paths(
    json: &str,
    field_configs: &HashMap<String, ValidationConfig>,
) -> Result<(), Vec<JsonValidationError>> {
    let json_value: Value = serde_json::from_str(json).map_err(|e| {
        vec![JsonValidationError::new(
            "root",
            format!("JSON parsing error: {}", e),
        )]
    })?;

    let mut all_errors = Vec::new();

    for (path, config) in field_configs {
        if let Some(field_value) = get_value_at_path(&json_value, path) {
            if let Err(errors) = validate_json_value(field_value, config) {
                // Prefix errors with the path
                for error in errors {
                    let full_path = if path == "." {
                        error.path
                    } else {
                        format!("{}.{}", path, error.path)
                    };
                    all_errors.push(JsonValidationError::new(full_path, error.message));
                }
            }
        } else {
            // Field not found
            all_errors.push(JsonValidationError::new(
                path,
                "Field is required but not found",
            ));
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

/// Get a value at a JSON path (simple dot notation)
fn get_value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "." {
        return Some(value);
    }

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(obj) => {
                current = obj.get(part)?;
            }
            _ => return None,
        }
    }

    Some(current)
}
