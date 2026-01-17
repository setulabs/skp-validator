//! ValidatedJson extractor for Axum.

use axum::extract::{FromRequest, Request, rejection::JsonRejection as AxumJsonRejection};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use skp_validator_core::{Validate, ValidationContext};
use std::ops::{Deref, DerefMut};

use crate::rejection::{JsonRejection, ValidationRejection, ValidatedJsonRejection};

/// Validated JSON extractor for Axum.
///
/// This extractor parses JSON from the request body and validates it
/// using the `Validate` trait. If parsing or validation fails, an
/// appropriate error response is returned.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{routing::post, Router};
/// use skp_validator_axum::ValidatedJson;
/// use skp_validator::Validate;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(required, length(min = 3))]
///     name: String,
///     #[validate(email)]
///     email: String,
/// }
///
/// async fn create_user(ValidatedJson(user): ValidatedJson<CreateUser>) -> String {
///     format!("Created user: {}", user.name)
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Consume the extractor and return the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for ValidatedJson<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidatedJsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Use Axum's built-in JSON extractor
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e: AxumJsonRejection| JsonRejection::JsonDataError(e.to_string()))?;

        // Validate
        let ctx = ValidationContext::default();
        if let Err(errors) = value.validate_with_context(&ctx) {
            return Err(ValidationRejection::new(errors).into());
        }

        Ok(ValidatedJson(value))
    }
}

impl<T: serde::Serialize> IntoResponse for ValidatedJson<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// Validated JSON with custom context.
///
/// Same as `ValidatedJson` but allows passing a custom `ValidationContext`.
#[derive(Debug, Clone)]
pub struct ValidatedJsonWithContext<T> {
    /// The validated value
    pub value: T,
    /// The validation context used
    pub context: ValidationContext,
}

impl<T> ValidatedJsonWithContext<T> {
    /// Create with a specific context
    pub fn with_context(value: T, context: ValidationContext) -> Self {
        Self { value, context }
    }
}

impl<T> Deref for ValidatedJsonWithContext<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, Clone)]
    struct TestUser {
        name: String,
        age: u32,
    }

    impl Validate for TestUser {
        fn validate_with_context(&self, _ctx: &ValidationContext) -> skp_validator_core::ValidationResult<()> {
            use skp_validator_core::{ValidationErrors, ValidationError};
            
            let mut errors = ValidationErrors::new();
            
            if self.name.trim().is_empty() {
                errors.add_field_error("name", 
                    ValidationError::new("name", "required", "Name is required"));
            }
            
            if self.age < 18 {
                errors.add_field_error("age",
                    ValidationError::new("age", "range.min", "Must be at least 18"));
            }
            
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        }
    }

    #[test]
    fn test_validated_json_deref() {
        let user = TestUser { name: "John".to_string(), age: 25 };
        let validated = ValidatedJson(user);
        
        assert_eq!(validated.name, "John");
        assert_eq!(validated.age, 25);
    }

    #[test]
    fn test_validated_json_into_inner() {
        let user = TestUser { name: "John".to_string(), age: 25 };
        let validated = ValidatedJson(user);
        
        let inner = validated.into_inner();
        assert_eq!(inner.name, "John");
    }
}
