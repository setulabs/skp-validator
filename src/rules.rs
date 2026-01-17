use crate::context::ValidationContext;
use crate::error::{FieldError, ValidationResult};
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A validation rule that can be applied to a field
#[derive(Clone)]
pub enum ValidationRule {
    /// Field must not be None/empty
    Required { message: Option<String> },

    /// String length validation
    Length {
        min: Option<usize>,
        max: Option<usize>,
        message: Option<String>,
    },

    /// Regular expression validation
    Regex {
        pattern: String,
        message: Option<String>,
    },

    /// Email format validation
    Email { message: Option<String> },

    /// Phone number format validation
    Phone { message: Option<String> },

    /// Numeric range validation
    Range {
        min: Option<f64>,
        max: Option<f64>,
        message: Option<String>,
    },

    /// Integer range validation
    IntRange {
        min: Option<i64>,
        max: Option<i64>,
        message: Option<String>,
    },

    /// Date validation with custom format
    Date {
        format: String,
        message: Option<String>,
    },

    /// Date range validation
    DateRange {
        min: Option<String>,
        max: Option<String>,
        format: String,
        message: Option<String>,
    },

    /// Age validation derived from date of birth
    Age {
        min: Option<u32>,
        max: Option<u32>,
        date_format: String,
        message: Option<String>,
    },

    /// Allowed values validation
    AllowedValues {
        values: Vec<String>,
        message: Option<String>,
    },

    /// Custom validation function
    Custom {
        validator: Arc<dyn Fn(&ValidationContext) -> ValidationResult<()> + Send + Sync>,
        message: Option<String>,
    },

    /// Contextual validation that depends on other fields
    Contextual {
        validator: Arc<dyn Fn(&ValidationContext) -> ValidationResult<()> + Send + Sync>,
        message: Option<String>,
    },

    /// Field transformation: convert to uppercase
    Uppercase,

    /// Field transformation: convert to lowercase
    Lowercase,

    /// Field transformation: trim whitespace
    Trim,
}

impl ValidationRule {
    /// Validate a value against this rule
    pub fn validate(
        &self,
        field_name: &str,
        value: Option<&Value>,
        context: &mut ValidationContext,
    ) -> ValidationResult<()> {
        match self {
            ValidationRule::Required { message } => {
                if value.is_none()
                    || value.unwrap().is_null()
                    || (value.unwrap().is_string()
                        && value.unwrap().as_str().unwrap().trim().is_empty())
                {
                    let msg = message
                        .clone()
                        .unwrap_or_else(|| format!("Field '{}' is required", field_name));
                    return Err(vec![FieldError::new(field_name, msg)]);
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
                                return Err(vec![FieldError::new(field_name, msg)]);
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
                                return Err(vec![FieldError::new(field_name, msg)]);
                            }
                        }
                    }
                }
            }

            ValidationRule::Regex { pattern, message } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        if let Ok(regex) = Regex::new(pattern) {
                            if !regex.is_match(s) {
                                let msg = message.clone().unwrap_or_else(|| {
                                    format!(
                                        "Field '{}' does not match required pattern",
                                        field_name
                                    )
                                });
                                return Err(vec![FieldError::new(field_name, msg)]);
                            }
                        }
                    }
                }
            }

            ValidationRule::Email { message } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        // Simple email regex - in production you might want a more comprehensive one
                        let email_regex = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
                        if !email_regex.is_match(s) {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be a valid email address", field_name)
                            });
                            return Err(vec![FieldError::new(field_name, msg)]);
                        }
                    }
                }
            }

            ValidationRule::Phone { message } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        // Simple phone regex - adjust as needed
                        let phone_regex = Regex::new(r"^\+?[\d\s\-\(\)]+$").unwrap();
                        if !phone_regex.is_match(s) {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!("Field '{}' must be a valid phone number", field_name)
                            });
                            return Err(vec![FieldError::new(field_name, msg)]);
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
                                return Err(vec![FieldError::new(field_name, msg)]);
                            }
                        }
                        if let Some(max_val) = max {
                            if num > *max_val {
                                let msg = message.clone().unwrap_or_else(|| {
                                    format!("Field '{}' must be at most {}", field_name, max_val)
                                });
                                return Err(vec![FieldError::new(field_name, msg)]);
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
                                return Err(vec![FieldError::new(field_name, msg)]);
                            }
                        }
                        if let Some(max_val) = max {
                            if num > *max_val {
                                let msg = message.clone().unwrap_or_else(|| {
                                    format!("Field '{}' must be at most {}", field_name, max_val)
                                });
                                return Err(vec![FieldError::new(field_name, msg)]);
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
                        return Err(vec![FieldError::new(field_name, msg)]);
                    }
                }
            }

            ValidationRule::Date { format, message } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        if NaiveDate::parse_from_str(s, format).is_err() {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Field '{}' must be a valid date in format {}",
                                    field_name, format
                                )
                            });
                            return Err(vec![FieldError::new(field_name, msg)]);
                        }
                    }
                }
            }

            ValidationRule::DateRange {
                min,
                max,
                format,
                message,
            } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        if let Ok(date) = NaiveDate::parse_from_str(s, format) {
                            if let Some(min_date_str) = min {
                                if let Ok(min_date) =
                                    NaiveDate::parse_from_str(min_date_str, format)
                                {
                                    if date < min_date {
                                        let msg = message.clone().unwrap_or_else(|| {
                                            format!(
                                                "Field '{}' must be on or after {}",
                                                field_name, min_date_str
                                            )
                                        });
                                        return Err(vec![FieldError::new(field_name, msg)]);
                                    }
                                }
                            }
                            if let Some(max_date_str) = max {
                                if let Ok(max_date) =
                                    NaiveDate::parse_from_str(max_date_str, format)
                                {
                                    if date > max_date {
                                        let msg = message.clone().unwrap_or_else(|| {
                                            format!(
                                                "Field '{}' must be on or before {}",
                                                field_name, max_date_str
                                            )
                                        });
                                        return Err(vec![FieldError::new(field_name, msg)]);
                                    }
                                }
                            }
                        } else {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Field '{}' must be a valid date in format {}",
                                    field_name, format
                                )
                            });
                            return Err(vec![FieldError::new(field_name, msg)]);
                        }
                    }
                }
            }

            ValidationRule::Age {
                min,
                max,
                date_format,
                message,
            } => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        if let Ok(dob) = NaiveDate::parse_from_str(s, date_format) {
                            let today = chrono::Utc::now().date_naive();
                            let age = today.year()
                                - dob.year()
                                - if today.ordinal() < dob.ordinal() {
                                    1
                                } else {
                                    0
                                };

                            if let Some(min_age) = min {
                                if age < *min_age as i32 {
                                    let msg = message.clone().unwrap_or_else(|| {
                                        format!("Age must be at least {} years", min_age)
                                    });
                                    return Err(vec![FieldError::new(field_name, msg)]);
                                }
                            }
                            if let Some(max_age) = max {
                                if age > *max_age as i32 {
                                    let msg = message.clone().unwrap_or_else(|| {
                                        format!("Age must be at most {} years", max_age)
                                    });
                                    return Err(vec![FieldError::new(field_name, msg)]);
                                }
                            }
                        } else {
                            let msg = message.clone().unwrap_or_else(|| {
                                format!(
                                    "Field '{}' must be a valid date in format {}",
                                    field_name, date_format
                                )
                            });
                            return Err(vec![FieldError::new(field_name, msg)]);
                        }
                    }
                }
            }

            ValidationRule::Uppercase => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        let transformed = s.to_uppercase();
                        context.set_field_value(field_name, Value::String(transformed));
                    }
                }
            }

            ValidationRule::Lowercase => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        let transformed = s.to_lowercase();
                        context.set_field_value(field_name, Value::String(transformed));
                    }
                }
            }

            ValidationRule::Trim => {
                if let Some(val) = value {
                    if let Some(s) = val.as_str() {
                        let transformed = s.trim().to_string();
                        context.set_field_value(field_name, Value::String(transformed));
                    }
                }
            }

            ValidationRule::Custom { validator, message } => {
                if let Err(errors) = validator(context) {
                    if !errors.is_empty() {
                        let msg = message
                            .clone()
                            .unwrap_or_else(|| "Custom validation failed".to_string());
                        return Err(vec![FieldError::new(field_name, msg)]);
                    }
                }
            }

            ValidationRule::Contextual { validator, message } => {
                if let Err(errors) = validator(context) {
                    if !errors.is_empty() {
                        let msg = message
                            .clone()
                            .unwrap_or_else(|| "Contextual validation failed".to_string());
                        return Err(vec![FieldError::new(field_name, msg)]);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Configuration for validating a specific field
#[derive(Clone)]
pub struct FieldValidationConfig {
    /// The field name
    pub field_name: String,
    /// The validation rules to apply
    pub rules: Vec<ValidationRule>,
}

impl FieldValidationConfig {
    /// Create a new field validation config
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
            rules: Vec::new(),
        }
    }

    /// Add a validation rule
    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validate a field value
    pub fn validate(
        &self,
        value: Option<&serde_json::Value>,
        context: &mut ValidationContext,
    ) -> ValidationResult<()> {
        let mut all_errors = Vec::new();

        // First apply transformations
        for rule in &self.rules {
            match rule {
                ValidationRule::Uppercase | ValidationRule::Lowercase | ValidationRule::Trim => {
                    if let Err(errors) = rule.validate(&self.field_name, value, context) {
                        all_errors.extend(errors);
                    }
                }
                _ => {}
            }
        }

        // Get the potentially transformed value (clone to avoid borrowing issues)
        let transformed_value = context.get_field_value(&self.field_name).cloned();

        // Then validate other rules with the transformed value
        for rule in &self.rules {
            match rule {
                ValidationRule::Uppercase | ValidationRule::Lowercase | ValidationRule::Trim => {
                    // Already processed transformations
                }
                _ => {
                    if let Err(errors) =
                        rule.validate(&self.field_name, transformed_value.as_ref(), context)
                    {
                        all_errors.extend(errors);
                    }
                }
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}

/// Represents a field dependency where validation of one field depends on another field's value
#[derive(Clone)]
pub struct FieldDependency {
    /// The field that depends on another field
    pub dependent_field: String,
    /// The field that this dependency depends on
    pub depends_on_field: String,
    /// The condition that must be met for the dependency to apply
    pub condition: Arc<dyn Fn(&ValidationContext) -> bool + Send + Sync>,
    /// The validation rules to apply when the condition is met
    pub validation_rules: Vec<ValidationRule>,
    /// Optional description of the dependency
    pub description: Option<String>,
}

impl FieldDependency {
    /// Create a new field dependency
    pub fn new(
        dependent_field: impl Into<String>,
        depends_on_field: impl Into<String>,
        condition: Arc<dyn Fn(&ValidationContext) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            dependent_field: dependent_field.into(),
            depends_on_field: depends_on_field.into(),
            condition,
            validation_rules: Vec::new(),
            description: None,
        }
    }

    /// Add a validation rule to this dependency
    pub fn add_rule(mut self, rule: ValidationRule) -> Self {
        self.validation_rules.push(rule);
        self
    }

    /// Set a description for this dependency
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if the dependency condition is met
    pub fn condition_met(&self, context: &ValidationContext) -> bool {
        (self.condition)(context)
    }

    /// Validate the dependent field when condition is met
    pub fn validate(
        &self,
        value: Option<&Value>,
        context: &mut ValidationContext,
    ) -> ValidationResult<()> {
        let mut all_errors = Vec::new();

        for rule in &self.validation_rules {
            if let Err(errors) = rule.validate(&self.dependent_field, value, context) {
                all_errors.extend(errors);
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}

/// Configuration for validating an entire struct/type
#[derive(Clone)]
pub struct ValidationConfig {
    /// Field validation configurations
    pub fields: HashMap<String, FieldValidationConfig>,
    /// Field dependencies
    pub dependencies: Vec<FieldDependency>,
}

impl ValidationConfig {
    /// Create a new validation config
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add field validation config
    pub fn add_field(mut self, config: FieldValidationConfig) -> Self {
        self.fields.insert(config.field_name.clone(), config);
        self
    }

    /// Add a field dependency
    pub fn add_dependency(mut self, dependency: FieldDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Validate an object against this configuration
    pub fn validate(&self, context: &mut ValidationContext) -> ValidationResult<()> {
        let mut all_errors = Vec::new();

        // First validate all fields
        for (field_name, field_config) in &self.fields {
            let value = context.get_field_value(field_name).cloned();
            if let Err(errors) = field_config.validate(value.as_ref(), context) {
                all_errors.extend(errors);
            }
        }

        // Then validate dependencies
        for dependency in &self.dependencies {
            if dependency.condition_met(context) {
                let field_name = &dependency.dependent_field;
                let value = context.get_field_value(field_name).cloned();
                if let Err(errors) = dependency.validate(value.as_ref(), context) {
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
}
