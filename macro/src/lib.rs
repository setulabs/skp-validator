use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro for making a struct validatable
///
/// # Example
/// ```ignore
/// use oc_validator::Validate;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
/// struct User {
///     #[validate]
///     pub name: String,
///
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Validate can only be derived for structs with named fields"),
        },
        _ => panic!("Validate can only be derived for structs"),
    };

    let field_validations = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let has_validate_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("validate"));

        if has_validate_attr {
            quote! {
                let field_config = oc_validator::FieldValidationConfig::new(#field_name_str)
                    .add_rule(oc_validator::ValidationRule::Required { message: None });

                config = config.add_field(field_config);
            }
        } else {
            quote! {}
        }
    });

    let expanded = quote! {
        impl oc_validator::Validate for #name {
            fn validate(&self) -> oc_validator::ValidationResult<()> {
                let config = Self::validation_config();
                let mut context = oc_validator::ValidationContext::from_serde(self)
                    .map_err(|e| vec![oc_validator::FieldError::new("serialization", format!("Serialization error: {}", e))])?;
                config.validate(&mut context)
            }

            fn validate_with_transform(&self) -> oc_validator::ValidationResult<Self>
            where
                Self: Clone + Sized,
            {
                let config = Self::validation_config();
                let mut context = oc_validator::ValidationContext::from_serde(self)
                    .map_err(|e| vec![oc_validator::FieldError::new("serialization", format!("Serialization error: {}", e))])?;
                config.validate(&mut context)?;

                // Apply transformations back to the struct
                let mut transformed = self.clone();
                // Note: For now, transformations are applied during validation but not persisted back
                // This would require more complex macro logic to modify struct fields
                Ok(transformed)
            }

            fn validation_config() -> oc_validator::ValidationConfig {
                let mut config = oc_validator::ValidationConfig::new();

                #(#field_validations)*

                config
            }
        }
    };

    TokenStream::from(expanded)
}
