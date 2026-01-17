//! Attribute parser for validation rules.

use syn::{Attribute, Lit, Result, Error};
use syn::Token;
use syn::punctuated::Punctuated;
use syn::parse::Parse;

/// Parsed validation rule.
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Skip validation for this field
    Skip,
    /// Field is required
    Required { message: Option<String> },
    /// String length constraints
    Length { min: Option<usize>, max: Option<usize>, equal: Option<usize>, message: Option<String> },
    /// Numeric range constraints (supports Int/Float literals)
    Range { min: Option<Lit>, max: Option<Lit>, message: Option<String> },
    /// Email format
    Email { message: Option<String> },
    /// URL format
    Url { message: Option<String> },
    /// IP address
    Ip { version: Option<String>, message: Option<String> },
    /// UUID
    Uuid { version: Option<usize>, message: Option<String> },
    /// Phone number
    Phone { message: Option<String> },
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
    MultipleOf { value: Lit, message: Option<String> },
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
    /// Trim validation (must be trimmed)
    Trim { message: Option<String> },
    /// Uppercase validation (must be uppercase)
    Uppercase { message: Option<String> },
    /// Lowercase validation (must be lowercase)
    Lowercase { message: Option<String> },
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
        if path.is_ident("skip") { rules.push(ValidationRule::Skip); return Ok(()); }
        if path.is_ident("nested") { rules.push(ValidationRule::Nested); return Ok(()); }
        if path.is_ident("dive") { rules.push(ValidationRule::Dive); return Ok(()); }
        
        if path.is_ident("trim") { 
            rules.push(ValidationRule::Trim { message: parse_message_arg(&meta)? }); 
            return Ok(()); 
        }
        if path.is_ident("uppercase") { 
            rules.push(ValidationRule::Uppercase { message: parse_message_arg(&meta)? }); 
            return Ok(()); 
        }
        if path.is_ident("lowercase") { 
            rules.push(ValidationRule::Lowercase { message: parse_message_arg(&meta)? }); 
            return Ok(()); 
        }
        
        if path.is_ident("required") {
            rules.push(ValidationRule::Required { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("email") {
            rules.push(ValidationRule::Email { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("url") {
            rules.push(ValidationRule::Url { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("ascii") {
            rules.push(ValidationRule::Ascii { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("alphanumeric") {
            rules.push(ValidationRule::Alphanumeric { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("unique_items") {
            rules.push(ValidationRule::UniqueItems { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("credit_card") {
            rules.push(ValidationRule::CreditCard { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        if path.is_ident("phone") {
            rules.push(ValidationRule::Phone { message: parse_message_arg(&meta)? });
            return Ok(());
        }
        
        if path.is_ident("ip") {
            let mut version = None;
            let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("version") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Str(s) = lit { version = Some(s.value()); }
                    } else if nested.path.is_ident("message") {
                         message = parse_param_string(&nested)?;
                    }
                    Ok(())
                })?;
            } else { message = parse_message_arg(&meta)?; }
            rules.push(ValidationRule::Ip { version, message });
            return Ok(());
        }
        
        if path.is_ident("uuid") {
            let mut version = None;
            let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("version") {
                        nested.input.parse::<Token![=]>()?;
                        let lit: Lit = nested.input.parse()?;
                        if let Lit::Int(i) = lit { version = Some(i.base10_parse()?); }
                    } else if nested.path.is_ident("message") {
                         message = parse_param_string(&nested)?;
                    }
                    Ok(())
                })?;
            } else { message = parse_message_arg(&meta)?; }
            rules.push(ValidationRule::Uuid { version, message });
            return Ok(());
        }
        
        if path.is_ident("length") {
            let mut min = None; let mut max = None; let mut equal = None; let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") { min = Some(parse_param_usize(&nested)?); }
                    else if nested.path.is_ident("max") { max = Some(parse_param_usize(&nested)?); }
                    else if nested.path.is_ident("equal") { equal = Some(parse_param_usize(&nested)?); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Length { min, max, equal, message });
            return Ok(());
        }
        
        if path.is_ident("range") {
            let mut min = None; let mut max = None; let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") { min = Some(parse_param_lit(&nested)?); }
                    else if nested.path.is_ident("max") { max = Some(parse_param_lit(&nested)?); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Range { min, max, message });
            return Ok(());
        }
        
        if path.is_ident("pattern") {
            let mut regex = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("regex") { regex = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Pattern { regex, message });
            return Ok(());
        }
        
        if path.is_ident("must_match") {
            let mut other = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("other") { other = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::MustMatch { other, message });
            return Ok(());
        }
        
        if path.is_ident("contains") {
            let mut value = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("value") { value = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Contains { value, message });
            return Ok(());
        }

        if path.is_ident("prefix") {
            let mut value = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("value") { value = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Prefix { value, message });
            return Ok(());
        }

        if path.is_ident("suffix") {
            let mut value = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("value") { value = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::Suffix { value, message });
            return Ok(());
        }
        
        if path.is_ident("multiple_of") {
            // Default to 1 (Int) but overwritten
            let mut value = Lit::Int(syn::LitInt::new("1", proc_macro2::Span::call_site()));
            let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("value") { value = parse_param_lit(&nested)?; }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
                    Ok(())
                })?;
            }
            rules.push(ValidationRule::MultipleOf { value, message });
            return Ok(());
        }
        
        if path.is_ident("allowed_values") {
             let mut values = Vec::new();
             let mut message = None;
             if meta.input.peek(syn::token::Paren) {
                 meta.parse_nested_meta(|nested| {
                      if nested.path.is_ident("value") {
                           if let Some(s) = parse_param_string(&nested)? { values.push(s); }
                      } else if nested.path.is_ident("values") {
                           nested.input.parse::<Token![=]>()?;
                           let content;
                           syn::bracketed!(content in nested.input);
                           let list: Punctuated<Lit, Token![,]> = content.parse_terminated(Lit::parse, Token![,])?;
                           for lit in list {
                               if let Lit::Str(s) = lit { values.push(s.value()); }
                           }
                      } else if nested.path.is_ident("message") {
                           message = parse_param_string(&nested)?;
                      }
                      Ok(())
                 })?;
             }
             rules.push(ValidationRule::AllowedValues { values, message });
             return Ok(());
        }

        if path.is_ident("custom") {
            let mut function = String::new(); let mut message = None;
            if meta.input.peek(syn::token::Paren) {
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("function") { function = parse_param_string(&nested)?.unwrap_or_default(); }
                    else if nested.path.is_ident("message") { message = parse_param_string(&nested)?; }
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

/// Try to parse a message argument from a rule if present (e.g. `email(message = "...")`) or `email` (no args)
fn parse_message_arg(meta: &syn::meta::ParseNestedMeta) -> Result<Option<String>> {
    if meta.input.peek(syn::token::Paren) {
        let mut message = None;
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("message") {
                message = parse_param_string(&nested)?;
            }
            Ok(())
        })?;
        Ok(message)
    } else {
        Ok(None)
    }
}

fn parse_param_string(meta: &syn::meta::ParseNestedMeta) -> Result<Option<String>> {
    meta.input.parse::<Token![=]>()?;
    let lit: Lit = meta.input.parse()?;
    if let Lit::Str(s) = lit {
        Ok(Some(s.value()))
    } else {
        Ok(None)
    }
}

fn parse_param_usize(meta: &syn::meta::ParseNestedMeta) -> Result<usize> {
    meta.input.parse::<Token![=]>()?;
    let lit: Lit = meta.input.parse()?;
    if let Lit::Int(i) = lit {
        i.base10_parse()
    } else {
        Err(meta.error("expected integer"))
    }
}

fn parse_param_lit(meta: &syn::meta::ParseNestedMeta) -> Result<Lit> {
    meta.input.parse::<Token![=]>()?;
    meta.input.parse()
}
