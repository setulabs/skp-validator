//! String validation rules.

pub mod length;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "regex")]
pub mod pattern;

#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "ip")]
pub mod ip;

#[cfg(feature = "uuid")]
pub mod uuid_rule;

#[cfg(feature = "phone")]
pub mod phone;

pub mod ascii;
pub mod alphanumeric;
pub mod contains;
