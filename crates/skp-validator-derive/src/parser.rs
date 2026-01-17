//! Attribute parser for validation rules.

use syn::{Attribute, Lit, Result, Error};
use syn::Token;

/// Parsed validation rule.
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Skip validation for this field
    Skip,
    /// Field is required
    Required { message: Option<String> },
    /// String length constraints
    Length { min: Option<usize>, max: Option<usize>, equal: Option<usize>, message: Option<String> },
    /// Numeric range constraints
    Range { min: Option<i64>, max: Option<i64>, message: Option<String> },
    /// Email format
    Email { message: Option<String> },
    /// URL format
    Url { message: Option<String> },
    /// Regex pattern
    Pattern { regex: String, message: Option<String> },
    /// ASCII only
    Ascii { message: Option<String> },
    /// Alphanumeric only
    Alphanumeric { message: Option<String> },
    /// Contains substring
    Contains { value: String, message: Option<String> },
    /// Starts with prefix
    Prefix { value: String, message: Option<String> },
    /// Ends with suffix
    Suffix { value: String, message: Option<String> },
    /// Multiple of
    MultipleOf { value: i64, message: Option<String> },
    /// Unique items in collection
    UniqueItems { message: Option<String> },
    /// Credit card validation
    CreditCard { message: Option<String> },
    /// Must match another field
    MustMatch { other: String, message: Option<String> },
    /// Allowed values
    AllowedValues { values: Vec<String>, message: Option<String> },
    /// Nested validation
    Nested,
    /// Dive into collection
    Dive,
    /// Trim whitespace
    Trim,
    /// Convert to uppercase
    Uppercase,
    /// Convert to lowercase
    Lowercase,
    /// Custom validation function
    Custom { function: String, message: Option<String> },
}

/// Parse a #[validate(...)] attribute into a list of rules.
pub fn parse_validate_attribute(attr: &Attribute) -> Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();
    
    // Parse the nested meta list: validate(rule1, rule2, ...)
    attr.parse_nested_meta(|meta| {
        let path = &meta.path;
        
        // Simple flags
        if path.is_ident("skip") {
            rules.push(ValidationRule::Skip);
            return Ok(());
        }
        if path.is_ident("required") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::Required { message });
            return Ok(());
        }
        if path.is_ident("email") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::Email { message });
            return Ok(());
        }
        if path.is_ident("url") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::Url { message });
            return Ok(());
        }
        if path.is_ident("ascii") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::Ascii { message });
            return Ok(());
        }
        if path.is_ident("alphanumeric") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::Alphanumeric { message });
            return Ok(());
        }
        if path.is_ident("unique_items") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::UniqueItems { message });
            return Ok(());
        }
        if path.is_ident("credit_card") {
            let message = parse_message_arg(&meta)?;
            rules.push(ValidationRule::CreditCard { message });
            return Ok(());
        }
        if path.is_ident("nested") {
            rules.push(ValidationRule::Nested);
            return Ok(());
        }
        if path.is_ident("dive") {
            rules.push(ValidationRule::Dive);
            return Ok(());
        }
        if path.is_ident("trim") {
            rules.push(ValidationRule::Trim);
            return Ok(());
        }
        if path.is_ident("uppercase") {
            rules.push(ValidationRule::Uppercase);
            return Ok(());
        }
        if path.is_ident("lowercase") {
            rules.push(ValidationRule::Lowercase);
            return Ok(());
        }
        
        // Rules with arguments: length(min = 3, max = 50)
        if path.is_ident("length") {
            let mut min = None;
            let mut max = None;
            let mut equal = None;
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit {
                            min = Some(i.base10_parse::<usize>()?);
                        }
                    } else if nested.path.is_ident("max") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit {
                            max = Some(i.base10_parse::<usize>()?);
                        }
                    } else if nested.path.is_ident("equal") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit {
                            equal = Some(i.base10_parse::<usize>()?);
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::Length { min, max, equal, message });
            return Ok(());
        }
        
        if path.is_ident("range") {
            let mut min = None;
            let mut max = None;
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit {
                            min = Some(i.base10_parse::<i64>()?);
                        }
                    } else if nested.path.is_ident("max") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit {
                            max = Some(i.base10_parse::<i64>()?);
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::Range { min, max, message });
            return Ok(());
        }
        
        if path.is_ident("pattern") {
            let mut regex = String::new();
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("regex") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            regex = s.value();
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::Pattern { regex, message });
            return Ok(());
        }
        
        if path.is_ident("must_match") {
            let mut other = String::new();
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("other") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            other = s.value();
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::MustMatch { other, message });
            return Ok(());
        }
        
        if path.is_ident("contains") {
            let mut value = String::new();
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("value") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            value = s.value();
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::Contains { value, message });
            return Ok(());
        }
        
        if path.is_ident("custom") {
            let mut function = String::new();
            let mut message = None;
            
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("function") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            function = s.value();
                        }
                    } else if nested.path.is_ident("message") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit {
                            message = Some(s.value());
                        }
                    }
                    Ok(())
                })?;
            }
            
            rules.push(ValidationRule::Custom { function, message });
            return Ok(());
        }
        
        // Unknown rule
        Err(Error::new_spanned(path, format!("Unknown validation rule: {:?}", path.get_ident())))
    })?;
    
    Ok(rules)
}

/// Try to parse a message argument from a rule if present
fn parse_message_arg(meta: &syn::meta::ParseNestedMeta) -> Result<Option<String>> {
    if meta.input.peek(syn::token::Paren) {
        let mut message = None;
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("message") {
                nested.input.parse::<Token![=]>()?;
                let lit: Lit = nested.input.parse()?;
                if let Lit::Str(s) = lit {
                    message = Some(s.value());
                }
            }
            Ok(())
        })?;
        Ok(message)
    } else {
        Ok(None)
    }
}
