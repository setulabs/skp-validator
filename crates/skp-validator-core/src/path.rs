//! Field path tracking for nested validation errors.
//!
//! Provides [`FieldPath`] and [`PathSegment`] for building paths to fields
//! in nested structures, arrays, and maps.

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A segment of a field path.
///
/// Paths can navigate through:
/// - Named fields in structs
/// - Numeric indices in arrays/vectors
/// - String keys in maps
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathSegment {
    /// A named field (e.g., `user` in `user.email`)
    Field(String),
    /// An array/vector index (e.g., `[0]` in `items[0].name`)
    Index(usize),
    /// A map key (e.g., `["key"]` in `metadata["key"].value`)
    Key(String),
}

impl PathSegment {
    /// Create a field segment
    pub fn field(name: impl Into<String>) -> Self {
        Self::Field(name.into())
    }

    /// Create an index segment
    pub fn index(idx: usize) -> Self {
        Self::Index(idx)
    }

    /// Create a key segment
    pub fn key(key: impl Into<String>) -> Self {
        Self::Key(key.into())
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(name) => write!(f, "{}", name),
            Self::Index(idx) => write!(f, "[{}]", idx),
            Self::Key(key) => write!(f, "[\"{}\"]", key),
        }
    }
}

/// A path to a field in a nested structure.
///
/// # Example
///
/// ```rust
/// use skp_validator_core::FieldPath;
///
/// // Build path: user.addresses[0].city
/// let path = FieldPath::new()
///     .push_field("user")
///     .push_field("addresses")
///     .push_index(0)
///     .push_field("city");
///
/// assert_eq!(path.to_string(), "user.addresses[0].city");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldPath {
    segments: Vec<PathSegment>,
}

impl FieldPath {
    /// Create an empty field path
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a path from a single field name
    pub fn from_field(name: impl Into<String>) -> Self {
        Self {
            segments: vec![PathSegment::Field(name.into())],
        }
    }

    /// Check if the path is empty (root level)
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the number of segments
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Add a field segment and return self (builder pattern)
    pub fn push_field(mut self, name: impl Into<String>) -> Self {
        self.segments.push(PathSegment::Field(name.into()));
        self
    }

    /// Add an index segment and return self (builder pattern)
    pub fn push_index(mut self, idx: usize) -> Self {
        self.segments.push(PathSegment::Index(idx));
        self
    }

    /// Add a key segment and return self (builder pattern)
    pub fn push_key(mut self, key: impl Into<String>) -> Self {
        self.segments.push(PathSegment::Key(key.into()));
        self
    }

    /// Add a field segment in place
    pub fn append_field(&mut self, name: impl Into<String>) {
        self.segments.push(PathSegment::Field(name.into()));
    }

    /// Add an index segment in place
    pub fn append_index(&mut self, idx: usize) {
        self.segments.push(PathSegment::Index(idx));
    }

    /// Add a key segment in place
    pub fn append_key(&mut self, key: impl Into<String>) {
        self.segments.push(PathSegment::Key(key.into()));
    }

    /// Get the segments as a slice
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Get the parent path (without the last segment)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            None
        } else {
            let mut parent = self.clone();
            parent.segments.pop();
            Some(parent)
        }
    }

    /// Get the last segment (leaf field name)
    pub fn last(&self) -> Option<&PathSegment> {
        self.segments.last()
    }

    /// Get the field name of the last segment if it's a Field
    pub fn last_field_name(&self) -> Option<&str> {
        match self.last() {
            Some(PathSegment::Field(name)) => Some(name),
            _ => None,
        }
    }

    /// Create a child path with a field segment
    pub fn child_field(&self, name: impl Into<String>) -> Self {
        self.clone().push_field(name)
    }

    /// Create a child path with an index segment
    pub fn child_index(&self, idx: usize) -> Self {
        self.clone().push_index(idx)
    }

    /// Create a child path with a key segment
    pub fn child_key(&self, key: impl Into<String>) -> Self {
        self.clone().push_key(key)
    }

    /// Convert to a dot-notation string
    pub fn to_dot_notation(&self) -> String {
        self.to_string()
    }

    /// Convert to a JSON pointer string (RFC 6901)
    pub fn to_json_pointer(&self) -> String {
        if self.segments.is_empty() {
            return String::new();
        }

        let mut pointer = String::new();
        for segment in &self.segments {
            pointer.push('/');
            match segment {
                PathSegment::Field(name) => {
                    // Escape ~ and / per RFC 6901
                    let escaped = name.replace('~', "~0").replace('/', "~1");
                    pointer.push_str(&escaped);
                }
                PathSegment::Index(idx) => {
                    pointer.push_str(&idx.to_string());
                }
                PathSegment::Key(key) => {
                    let escaped = key.replace('~', "~0").replace('/', "~1");
                    pointer.push_str(&escaped);
                }
            }
        }
        pointer
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Field(name) => {
                    if i > 0 {
                        // Always add dot before a field if there's a previous segment
                        write!(f, ".")?;
                    }
                    write!(f, "{}", name)?;
                }
                PathSegment::Index(idx) => {
                    write!(f, "[{}]", idx)?;
                }
                PathSegment::Key(key) => {
                    write!(f, "[\"{}\"]", key)?;
                }
            }
        }
        Ok(())
    }
}

impl From<&str> for FieldPath {
    fn from(s: &str) -> Self {
        Self::from_field(s)
    }
}

impl From<String> for FieldPath {
    fn from(s: String) -> Self {
        Self::from_field(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let path = FieldPath::from_field("email");
        assert_eq!(path.to_string(), "email");
    }

    #[test]
    fn test_nested_path() {
        let path = FieldPath::new()
            .push_field("user")
            .push_field("address")
            .push_field("city");
        assert_eq!(path.to_string(), "user.address.city");
    }

    #[test]
    fn test_array_path() {
        let path = FieldPath::new()
            .push_field("items")
            .push_index(0)
            .push_field("name");
        assert_eq!(path.to_string(), "items[0].name");
    }

    #[test]
    fn test_map_path() {
        let path = FieldPath::new()
            .push_field("metadata")
            .push_key("custom")
            .push_field("value");
        assert_eq!(path.to_string(), "metadata[\"custom\"].value");
    }

    #[test]
    fn test_json_pointer() {
        let path = FieldPath::new()
            .push_field("user")
            .push_field("addresses")
            .push_index(0)
            .push_field("city");
        assert_eq!(path.to_json_pointer(), "/user/addresses/0/city");
    }

    #[test]
    fn test_parent() {
        let path = FieldPath::new().push_field("user").push_field("email");
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "user");
    }
}
