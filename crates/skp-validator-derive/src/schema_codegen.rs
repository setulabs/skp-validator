use proc_macro2::TokenStream;
use quote::quote;
use syn::{FieldsNamed, Ident, Generics};
use crate::parser::{ValidationRule, parse_validate_attribute};

pub fn generate_metadata_impl(
    name: &Ident,
    generics: &Generics,
    fields: &FieldsNamed,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let mut field_schema_generators = Vec::new();
    let mut nested_generators = Vec::new();
    
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_ty = &field.ty;
        
        // Find validate attribute
        if let Some(validate_attr) = field.attrs.iter().find(|attr| attr.path().is_ident("validate")) {
             if let Ok(rules) = parse_validate_attribute(validate_attr) {
                 if rules.iter().any(|r| matches!(r, ValidationRule::Skip)) {
                     continue;
                 }
                 
                 let is_nested = rules.iter().any(|r| matches!(r, ValidationRule::Nested));
                 
                 if is_nested {
                     nested_generators.push(quote! {
                         type_validation.nested.insert(
                             #field_name_str.to_string(),
                             <#field_ty as skp_validator_core::schema::ValidationMetadata>::get_validation_rules()
                         );
                     });
                 }
                 
                 // Generate field rules
                 let rules_code: Vec<_> = rules.iter().filter_map(generate_rule_schema).collect();
                 if !rules_code.is_empty() {
                     field_schema_generators.push(quote! {
                         type_validation.fields.insert(
                             #field_name_str.to_string(),
                             skp_validator_core::schema::FieldValidation {
                                 rules: vec![#(#rules_code),*],
                             }
                         );
                     });
                 }
             }
        }
    }
    
    quote! {
        impl #impl_generics skp_validator_core::schema::ValidationMetadata for #name #ty_generics #where_clause {
            fn get_validation_rules() -> skp_validator_core::schema::TypeValidation {
                let mut type_validation = skp_validator_core::schema::TypeValidation::new();
                
                #(#field_schema_generators)*
                #(#nested_generators)*
                
                type_validation
            }
        }
    }
}

fn generate_rule_schema(rule: &ValidationRule) -> Option<TokenStream> {
    match rule {
        ValidationRule::Required { .. } => Some(quote!(skp_validator_core::schema::RuleSchema::Required)),
        ValidationRule::Email { .. } => Some(quote!(skp_validator_core::schema::RuleSchema::Email)),
        ValidationRule::Url { .. } => Some(quote!(skp_validator_core::schema::RuleSchema::Url)),
        ValidationRule::Ip { version, .. } => {
            let v = quote_option_string(version);
            Some(quote!(skp_validator_core::schema::RuleSchema::Ip { version: #v }))
        },
        ValidationRule::Uuid { version, .. } => {
            let v = quote_option_usize(version);
            Some(quote!(skp_validator_core::schema::RuleSchema::Uuid { version: #v }))
        },
        ValidationRule::Phone { .. } => Some(quote!(skp_validator_core::schema::RuleSchema::Phone)),
        ValidationRule::CreditCard { .. } => Some(quote!(skp_validator_core::schema::RuleSchema::CreditCard)),
        ValidationRule::Range { min, max, .. } => {
             let min = quote_option_lit_as_f64(min);
             let max = quote_option_lit_as_f64(max);
             Some(quote! {
                 skp_validator_core::schema::RuleSchema::Range {
                     min: #min,
                     max: #max,
                     min_exclusive: None,
                     max_exclusive: None,
                  }
             })
        },
        ValidationRule::Length { min, max, equal, .. } => {
             let min = quote_option_usize_as_u64(min);
             let max = quote_option_usize_as_u64(max);
             let equal = quote_option_usize_as_u64(equal);
             Some(quote! {
                 skp_validator_core::schema::RuleSchema::Length {
                     min: #min,
                     max: #max,
                     equal: #equal,
                 }
             })
        },
        ValidationRule::Pattern { regex, .. } => Some(quote! {
            skp_validator_core::schema::RuleSchema::Pattern { regex: #regex.to_string() }
        }),
        ValidationRule::AllowedValues { values, .. } => {
            let value_tokens = values.iter().map(|v| quote!(#v.to_string()));
            Some(quote! {
                skp_validator_core::schema::RuleSchema::AllowedValues { values: vec![#(#value_tokens),*] }
            })
        },
        ValidationRule::MustMatch { other, .. } => {
            Some(quote! {
                skp_validator_core::schema::RuleSchema::MustMatch { other_field: #other.to_string() }
            })
        },
        ValidationRule::Custom { function, .. } => Some(quote! {
             skp_validator_core::schema::RuleSchema::Custom { name: #function.to_string() }
        }),
        _ => None
    }
}

fn quote_option_string(opt: &Option<String>) -> TokenStream {
    match opt {
        Some(v) => quote!(Some(#v.to_string())),
        None => quote!(None),
    }
}

fn quote_option_usize(opt: &Option<usize>) -> TokenStream {
    match opt {
        Some(v) => quote!(Some(#v)),
        None => quote!(None),
    }
}

fn quote_option_usize_as_u64(opt: &Option<usize>) -> TokenStream {
    match opt {
        Some(v) => { let v = *v as u64; quote!(Some(#v)) },
        None => quote!(None),
    }
}

fn quote_option_lit_as_f64(opt: &Option<syn::Lit>) -> TokenStream {
    match opt {
        Some(lit) => {
            match lit {
                syn::Lit::Int(i) => {
                    let val = i.base10_parse::<f64>().expect("Invalid number");
                    quote!(Some(#val))
                },
                syn::Lit::Float(f) => {
                     let val = f.base10_parse::<f64>().expect("Invalid number");
                     quote!(Some(#val))
                },
                _ => quote!(None), 
            }
        },
        None => quote!(None),
    }
}
