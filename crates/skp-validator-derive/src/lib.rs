use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use quote::quote;

mod parser;
mod schema_codegen;

use parser::{ValidationRule, parse_validate_attribute};
use schema_codegen::generate_metadata_impl;

/// Derive macro for implementing the Validate trait.
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
    
    // Generate schema metadata
    let fields_named = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let metadata_impl = generate_metadata_impl(name, generics, fields_named);
    
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
        
        #metadata_impl
    };
    
    TokenStream::from(expanded)
}

fn generate_field_validation(field: &syn::Field) -> Option<proc_macro2::TokenStream> {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    
    // Check validation attribute
    if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("validate")) {
        let rules = match parse_validate_attribute(attr) {
            Ok(rules) => rules,
            Err(e) => {
                let err_msg = e.to_string();
                return Some(quote! { compile_error!(#err_msg); });
            }
        };
        
        let is_option = is_option(&field.ty);
        let field_type = &field.ty;
        
        let validations: Vec<_> = rules.iter().filter_map(|rule| {
             generate_rule_validation(field_name, &field_name_str, field_type, rule, is_option)
        }).collect();
        
        Some(quote! {
            #(#validations)*
        })
    } else {
        None
    }
}

fn generate_rule_validation(
    field_name: &syn::Ident,
    field_name_str: &str,
    field_type: &syn::Type,
    rule: &ValidationRule,
    is_option: bool
) -> Option<proc_macro2::TokenStream> {
    match rule {
        ValidationRule::Skip => None,
        
        ValidationRule::Required { message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("field is required".to_string()));
            Some(quote! {
                if self.#field_name == <#field_type as Default>::default() {
                     errors.add_field_error(
                        #field_name_str, 
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "required", 
                            #error_message
                        )
                     );
                }
            })
        },
        

        ValidationRule::Nested => {
            Some(quote! {
                if let Err(mut nested_errors) = self.#field_name.validate_with_context(ctx) {
                    errors.add_nested_errors(#field_name_str, nested_errors);
                }
            })
        },
        
        ValidationRule::Dive => {
            Some(quote! {
                use skp_validator_core::ValidateDive;
                let path = skp_validator_core::FieldPath::from_field(#field_name_str);
                if let Err(dive_errors) = self.#field_name.validate_dive(&path, ctx) {
                    errors.merge(dive_errors);
                }
            })
        },

        _ => {
            let rule_check = generate_leaf_rule_check(rule, field_name, field_name_str);
            if let Some(check) = rule_check {
                if is_option {
                     Some(quote! {
                         if let Some(ref val) = self.#field_name {
                             #check
                         }
                     })
                } else {
                     Some(quote! {
                         let val = &self.#field_name;
                         #check
                     })
                }
            } else {
                None
            }
        }
    }
}

fn generate_leaf_rule_check(
    rule: &ValidationRule,
    _field_ident: &syn::Ident,
    field_name_str: &str
) -> Option<proc_macro2::TokenStream> {
    match rule {
        ValidationRule::Length { min, max, equal, message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("invalid length".to_string()));
             let min = quote_option_usize(min);
             let max = quote_option_usize(max);
             let equal = quote_option_usize(equal);
             Some(quote! {
                 let len = val.len();
                 let mut valid = true;
                 if let Some(m) = #min { if len < m { valid = false; } }
                 if let Some(m) = #max { if len > m { valid = false; } }
                 if let Some(e) = #equal { if len != e { valid = false; } }
                 if !valid {
                      errors.add_field_error(
                        #field_name_str, 
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "length", 
                            #error_message
                        )
                      );
                 }
             })
        },

        ValidationRule::Range { min, max, message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("value out of range".to_string()));
             
             let min_check = if let Some(m) = min {
                 quote! { if *val < (#m as _) { valid = false; } }
             } else {
                 quote! {}
             };
             
             let max_check = if let Some(m) = max {
                 quote! { if *val > (#m as _) { valid = false; } }
             } else {
                 quote! {}
             };

             Some(quote! {
                 let mut valid = true;
                 #min_check
                 #max_check
                 if !valid {
                      errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "range", 
                            #error_message
                        )
                      );
                 }
             })
        },

        ValidationRule::Email { message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("invalid email".to_string()));
             Some(quote! {
                 if !val.contains('@') {
                      errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "email", 
                            #error_message
                        )
                      );
                 }
             })
        },
        
        ValidationRule::Url { message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("invalid url".to_string()));
             Some(quote! {
                 if !val.starts_with("http") {
                      errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "url", 
                            #error_message
                        )
                      );
                 }
             })
        },

        ValidationRule::Ip { version, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid IP address".to_string()));
            let version = quote_option_string(version);
            Some(quote! {
                let val_str = val.to_string();
                if val_str.parse::<std::net::IpAddr>().is_err() {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "ip", 
                            #error_message
                        )
                     );
                } else if let Some(ver) = #version {
                     let ip: std::net::IpAddr = val_str.parse().unwrap();
                     if ver == "v4" && !ip.is_ipv4() {
                         errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "ip", 
                                "Expected IPv4".to_string()
                            )
                         );
                     } else if ver == "v6" && !ip.is_ipv6() {
                         errors.add_field_error(
                            #field_name_str,
                            skp_validator_core::ValidationError::new(
                                #field_name_str,
                                "ip", 
                                "Expected IPv6".to_string()
                            )
                         );
                     }
                }
            })
        },
        
        ValidationRule::Uuid { version, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid UUID".to_string()));
            let version = quote_option_usize(version);
            Some(quote! {
                use skp_validator_core::Rule;
                let mut rule = skp_validator::rules::UuidRule::new();
                if let Some(v) = #version {
                    rule = rule.version(v as u8);
                }
                rule = rule.message(#error_message);
                
                if let Err(mut e) = rule.validate(&val.to_string(), ctx) {
                     for err in e.errors {
                         errors.add_field_error(#field_name_str, err);
                     }
                }
            })
        },

        ValidationRule::Phone { message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid phone number".to_string()));
            Some(quote! {
                 use skp_validator_core::Rule;
                 let rule = skp_validator::rules::PhoneRule::new().message(#error_message);
                 if let Err(mut e) = rule.validate(&val.to_string(), ctx) {
                     for err in e.errors {
                         errors.add_field_error(#field_name_str, err);
                     }
                 }
            })
        },
        
        ValidationRule::Prefix { value, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid prefix".to_string()));
            Some(quote! {
                if !val.starts_with(#value) {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "prefix", 
                            #error_message
                        )
                     );
                }
            })
        },

        ValidationRule::Suffix { value, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid suffix".to_string()));
            Some(quote! {
                if !val.ends_with(#value) {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "suffix", 
                            #error_message
                        )
                     );
                }
            })
        },

        ValidationRule::Contains { value, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Must contain value".to_string()));
            Some(quote! {
                if !val.contains(#value) {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "contains", 
                            #error_message
                        )
                     );
                }
            })
        },
        
        ValidationRule::Trim { message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Must be trimmed".to_string()));
            Some(quote! {
                if val.trim() != val {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "trim", 
                            #error_message
                        )
                     );
                }
            })
        },
        
        ValidationRule::Uppercase { message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Must be uppercase".to_string()));
            Some(quote! {
                if val.chars().any(|c| c.is_lowercase()) {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "uppercase", 
                            #error_message
                        )
                     );
                }
            })
        },

        ValidationRule::Lowercase { message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Must be lowercase".to_string()));
             Some(quote! {
                 if val.chars().any(|c| c.is_uppercase()) {
                      errors.add_field_error(
                         #field_name_str,
                         skp_validator_core::ValidationError::new(
                             #field_name_str,
                             "lowercase", 
                             #error_message
                         )
                      );
                 }
             })
        },

        ValidationRule::MultipleOf { value, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Not multiple of value".to_string()));
            Some(quote! {
                if val % (#value as _) != 0 {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "multiple_of", 
                            #error_message
                        )
                     );
                }
            })
        },
        
        ValidationRule::AllowedValues { values, message } => {
            let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Value not allowed".to_string()));
            let value_tokens = values.iter().map(|v| quote!(#v));
            Some(quote! {
                let allowed = vec![#(#value_tokens),*];
                if !allowed.contains(&val.to_string().as_str()) {
                     errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "allowed_values", 
                            #error_message
                        )
                     );
                }
            })
        },
        
        ValidationRule::MustMatch { other, message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Field mismatch".to_string()));
             let other_ident = syn::Ident::new(other, proc_macro2::Span::call_site());
             Some(quote! {
                 if val != &self.#other_ident {
                      errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "must_match", 
                            #error_message
                        )
                      );
                 }
             })
        },
        
        ValidationRule::CreditCard { message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid credit card number".to_string()));
             Some(quote! {
                 let val_str = val.to_string();
                 let mut sum = 0;
                 let mut double = false;
                 let mut valid = true;
                 for c in val_str.chars().rev() {
                     if let Some(mut digit) = c.to_digit(10) {
                         if double {
                             digit *= 2;
                             if digit > 9 { digit -= 9; }
                         }
                         sum += digit;
                         double = !double;
                     } else {
                         valid = false;
                         break;
                     }
                 }
                 if !valid || sum % 10 != 0 {
                      errors.add_field_error(
                        #field_name_str,
                        skp_validator_core::ValidationError::new(
                            #field_name_str,
                            "credit_card", 
                            #error_message
                        )
                      );
                 }
             })
        },
        
        ValidationRule::Pattern { regex, message } => {
             let error_message = message.as_ref().map(|m| quote!(#m.to_string())).unwrap_or_else(|| quote!("Invalid format".to_string()));
             Some(quote! {
                 use skp_validator_core::Rule;
                 let rule = skp_validator::rules::PatternRule::new(#regex).message(#error_message);
                 if let Err(mut e) = rule.validate(&val.to_string(), ctx) {
                     for err in e.errors {
                         errors.add_field_error(#field_name_str, err);
                     }
                 }
             })
        },
        
        ValidationRule::Custom { function, message } => {
            let function_path: syn::Path = syn::parse_str(function).expect("Invalid function path");
            let message_override = if let Some(msg) = message {
                quote! { e.message = #msg.to_string(); }
            } else {
                quote! {}
            };
            Some(quote! {
                if let Err(mut e) = #function_path(&val) {
                    #message_override
                    errors.add_field_error(#field_name_str, e);
                }
            })
        },
        
        _ => None
    }
}

fn quote_option_usize(opt: &Option<usize>) -> proc_macro2::TokenStream {
    match opt {
        Some(v) => quote!(Some(#v as usize)),
        None => quote!(None::<usize>),
    }
}

fn quote_option_string(opt: &Option<String>) -> proc_macro2::TokenStream {
    match opt {
        Some(v) => quote!(Some(#v)),
        None => quote!(None::<String>),
    }
}

fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
         if let Some(segment) = type_path.path.segments.last() {
             if segment.ident == "Option" {
                 return true;
             }
         }
    }
    false
}
