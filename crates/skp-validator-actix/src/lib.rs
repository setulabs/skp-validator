use actix_web::{FromRequest, HttpRequest, dev::Payload, Error};
use futures_util::future::{LocalBoxFuture, FutureExt};
use serde::de::DeserializeOwned;
use skp_validator_core::Validate;
use std::ops::{Deref, DerefMut};
use std::fmt;

/// Extractor that deserializes JSON and validates it.
///
/// Wraps `actix_web::web::Json` and performs validation on the deserialized struct.
/// If validation fails, returns `400 Bad Request` with the validation errors.
///
/// # Example
///
/// ```rust,ignore
/// use skp_validator_actix::ValidatedJson;
/// use skp_validator::Validate;
/// use serde::Deserialize;
/// use actix_web::{post, Responder};
///
/// #[derive(Debug, Deserialize, Validate)]
/// struct User {
///     #[validate(length(min = 3))]
///     name: String,
/// }
///
/// #[post("/users")]
/// async fn create_user(user: ValidatedJson<User>) -> impl Responder {
///     format!("Welcome {}!", user.name)
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Unwrap into inner T
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Display for ValidatedJson<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // Delegate to actix_web::web::Json extractor
        let fut = actix_web::web::Json::<T>::from_request(req, payload);

        async move {
            let json = fut.await?;
            let inner = json.into_inner();
            
            if let Err(e) = inner.validate() {
                // Return 400 Bad Request with validation errors body
                // Assuming serde feature is enabled on skp-validator-core (via workspace)
                // If not, fallback to string
                #[cfg(feature = "serde")]
                let body = e.to_json_string(); 
                
                #[cfg(not(feature = "serde"))]
                let body = e.to_string();

                // Note: to_json_string is only available if serde is enabled in core.
                // But my `ValidatedJson` extractor doesn't *know* if core has serde enabled unless I check it?
                // Actually `e.to_json_string()` is conditionally compiled in `error.rs` (Step 520).
                // So if core has serde, method exists.
                // If core doesn't, method doesn't exist.
                // So I need to use the method only if standard traits available?
                // Or I can just format it using Display if to_json_string fails to compile?
                // But I can't check feature of dependency easily in code like this without `cfg` check on local feature.
                // I will add `serde` feature to `skp-validator-actix` and propagate it to core.
                
                return Err(actix_web::error::ErrorBadRequest(body));
            }
            
            Ok(ValidatedJson(inner))
        }
        .boxed_local()
    }
}
