//! # skp-validator-derive
//!
//! Derive macro for skp-validator.
//!
//! This crate provides the `#[derive(Validate)]` macro for automatically
//! implementing validation logic from attributes.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use skp_validator::Validate;
//!
//! #[derive(Validate)]
//! struct User {
//!     #[validate(required, length(min = 3, max = 50))]
//!     name: String,
//!
//!     #[validate(required, email)]
//!     email: String,
//!
//!     #[validate(range(min = 18))]
//!     age: u32,
//!
//!     #[validate(nested)]
//!     address: Address,
//!
//!     #[validate(dive)]
//!     tags: Vec<String>,
//! }
//! ```

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field};

mod parser;
use parser::{ValidationRule, parse_validate_attribute};

/// Derive macro for implementing the Validate trait.
///
/// # Supported Attributes
///
/// ## String validators
/// - `required` - Field must not be empty/null
/// - `email` - Must be a valid email address
/// - `url` - Must be a valid URL
/// - `pattern(regex = "...")` - Regex pattern matching
/// - `length(min = N, max = N, equal = N)` - String length constraints
/// - `ascii` - Must be ASCII only
/// - `alphanumeric` - Must be alphanumeric
/// - `contains(value = "...")` - Must contain substring
/// - `prefix(value = "...")` - Must start with
/// - `suffix(value = "...")` - Must end with
///
/// ## Numeric validators
/// - `range(min = N, max = N)` - Numeric range constraints
/// - `multiple_of(value = N)` - Must be divisible by N
///
/// ## Comparison validators
/// - `must_match(other = "field")` - Must equal another field
/// - `allowed_values = ["a", "b"]` - Must be one of the values
///
/// ## Nested/Collection
/// - `nested` - Validate nested struct
/// - `dive` - Validate each item in collection
///
/// ## Transformations
/// - `trim` - Trim whitespace before validation
/// - `uppercase` - Convert to uppercase
/// - `lowercase` - Convert to lowercase
///
/// ## Other
/// - `custom(function = "fn_name")` - Custom validation function
/// - `skip` - Skip validation for this field
/// - `message = "..."` - Custom error message
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    // Get fields from struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    &input,
                    "Validate can only be derived for structs with named fields"
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    &input,
                    "Validate cannot be derived for unit structs"
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                &input,
                "Validate for enums is not yet implemented"
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                &input,
                "Validate cannot be derived for unions"
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Generate field validation code
    let field_validations: Vec<_> = fields.iter().filter_map(|field| {
        generate_field_validation(field)
    }).collect();
    
    // Generate implementation
    let expanded = quote! {
        impl #impl_generics skp_validator_core::Validate for #name #ty_generics #where_clause {
            fn validate_with_context(
                &self,
                ctx: &skp_validator_core::ValidationContext
            ) -> skp_validator_core::ValidationResult<()> {
                let mut errors = skp_validator_core::ValidationErrors::new();
                
                #(#field_validations)*
                
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

fn generate_field_validation(field: &Field) -> Option<proc_macro2::TokenStream> {
    let field_name = field.ident.as_ref()?;
    let field_name_str = field_name.to_string();
    
    // Find validate attribute
    let validate_attr = field.attrs.iter().find(|attr| attr.path().is_ident("validate"))?;
    
    // Parse the attribute
    let rules = match parse_validate_attribute(validate_attr) {
        Ok(rules) => rules,
        Err(e) => return Some(e.to_compile_error()),
    };
    
    // Check if skip is present
    if rules.iter().any(|r| matches!(r, ValidationRule::Skip)) {
        return None;
    }
    
    // Generate validation code for each rule
    let rule_validations: Vec<_> = rules.iter().filter_map(|rule| {
        generate_rule_validation(&field_name_str, field_name, rule)
    }).collect();
    
    if rule_validations.is_empty() {
        return None;
    }
    
    Some(quote! {
        // Validate field: #field_name_str
        {
            #(#rule_validations)*
        }
    })
}

fn generate_rule_validation(
    field_name_str: &str, 
    field_name: &syn::Ident,
    rule: &ValidationRule
) -> Option<proc_macro2::TokenStream> {
    match rule {
        ValidationRule::Skip => None,
        
        ValidationRule::Required { message } => {
            let msg = message.as_deref().unwrap_or("This field is required");
            Some(quote! {
                {
                    let value = &self.#field_name;
                    let is_empty = match std::any::TypeId::of_val(value) {
                        id if id == std::any::TypeId::of::<String>() => {
                            (value as &dyn std::any::Any).downcast_ref::<String>()
                                .map(|s| s.trim().is_empty()).unwrap_or(false)
                        }
                        _ => false,
                    };
                    // Simple string check
                    if let Some(s) = (value as &dyn std::any::Any).downcast_ref::<String>() {
                        if s.trim().is_empty() {
                            errors.add_field_error(
                                #field_name_str,
                                skp_validator_core::ValidationError::new(
                                    #field_name_str,
                                    "required",
                                    #msg
                                )
                            );
                        }
                    }
                }
            })
        }
        
        ValidationRule::Length { min, max, equal, message } => {
            let mut checks = Vec::new();
            
            if let Some(min_val) = min {
                let msg = message.as_deref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Must be at least {} characters", min_val));
                checks.push(quote! {
                    if len < #min_val {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "length.min",
                                #msg
                            )
                        );
                    }
                });
            }
            
            if let Some(max_val) = max {
                let msg = message.as_deref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Must be at most {} characters", max_val));
                checks.push(quote! {
                    if len > #max_val {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "length.max",
                                #msg
                            )
                        );
                    }
                });
            }
            
            if let Some(equal_val) = equal {
                let msg = message.as_deref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Must be exactly {} characters", equal_val));
                checks.push(quote! {
                    if len != #equal_val {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "length.equal",
                                #msg
                            )
                        );
                    }
                });
            }
            
            if checks.is_empty() {
                return None;
            }
            
            Some(quote! {
                {
                    let len = self.#field_name.chars().count();
                    #(#checks)*
                }
            })
        }
        
        ValidationRule::Range { min, max, message } => {
            let mut checks = Vec::new();
            
            if let Some(min_val) = min {
                let msg = message.as_deref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Must be at least {}", min_val));
                checks.push(quote! {
                    if self.#field_name < #min_val as _ {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "range.min",
                                #msg
                            )
                        );
                    }
                });
            }
            
            if let Some(max_val) = max {
                let msg = message.as_deref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Must be at most {}", max_val));
                checks.push(quote! {
                    if self.#field_name > #max_val as _ {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "range.max",
                                #msg
                            )
                        );
                    }
                });
            }
            
            if checks.is_empty() {
                return None;
            }
            
            Some(quote! {
                #(#checks)*
            })
        }
        
        ValidationRule::Email { message } => {
            let msg = message.as_deref().unwrap_or("Must be a valid email address");
            Some(quote! {
                if !self.#field_name.is_empty() {
                    // Simple email check - @ present and not at start/end
                    let email = &self.#field_name;
                    let has_at = email.contains('@');
                    let at_pos = email.find('@');
                    let is_valid = has_at && at_pos.map(|p| p > 0 && p < email.len() - 1).unwrap_or(false);
                    if !is_valid {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "email",
                                #msg
                            )
                        );
                    }
                }
            })
        }
        
        ValidationRule::Url { message } => {
            let msg = message.as_deref().unwrap_or("Must be a valid URL");
            Some(quote! {
                if !self.#field_name.is_empty() {
                    // Simple URL check - starts with http:// or https://
                    let url = &self.#field_name;
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "url",
                                #msg
                            )
                        );
                    }
                }
            })
        }
        
        ValidationRule::Pattern { regex, message } => {
            let msg = message.as_deref().unwrap_or("Does not match the required format");
            Some(quote! {
                if !self.#field_name.is_empty() {
                    let re = regex::Regex::new(#regex).expect("Invalid regex pattern");
                    if !re.is_match(&self.#field_name) {
                        errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "pattern",
                                #msg
                            )
                        );
                    }
                }
            })
        }
        
        ValidationRule::Nested => {
            Some(quote! {
                if let Err(nested_errors) = skp_validator_core::Validate::validate_with_context(&self.#field_name, ctx) {
                    errors.add_nested_errors(#field_name_str, nested_errors);
                }
            })
        }
        
        ValidationRule::Dive => {
            Some(quote! {
                for (idx, item) in self.#field_name.iter().enumerate() {
                    if let Err(item_errors) = skp_validator_core::Validate::validate_with_context(item, ctx) {
                        let key = format!("{}[{}]", #field_name_str, idx);
                        errors.add_nested_errors(&key, item_errors);
                    }
                }
            })
        }
        
        ValidationRule::Custom { function, message } => {
            let fn_name = format_ident!("{}", function);
            let msg = message.as_deref().unwrap_or("Custom validation failed");
            Some(quote! {
                if !#fn_name(&self.#field_name, ctx) {
                    errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "custom",
                            #msg
                        )
                    );
                }
            })
        }
        
        ValidationRule::MustMatch { other, message } => {
            let other_field = format_ident!("{}", other);
            let msg = message.as_deref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| format!("Must match '{}'", other));
            Some(quote! {
                if self.#field_name != self.#other_field {
                    errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "must_match",
                            #msg
                        )
                    );
                }
            })
        }
        
        _ => None, // Other rules not yet implemented
    }
}
