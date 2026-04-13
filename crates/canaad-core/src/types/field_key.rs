//! `FieldKey` newtype and extension key validation.

use crate::error::AadError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Reserved field names that cannot be used as extensions.
pub const RESERVED_KEYS: &[&str] = &["v", "tenant", "resource", "purpose", "ts"];

// human-readable pattern description used in `InvalidExtensionKeyFormat` errors
const EXTENSION_KEY_PATTERN: &str = "x_<app>_<field> where app is [a-z]+ and field is [a-z_]+";

/// Field key matching pattern `[a-z_]+`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldKey(String);

impl FieldKey {
    /// Creates a new `FieldKey`.
    ///
    /// # Errors
    ///
    /// - `EmptyFieldKey` if empty
    /// - `InvalidFieldKey` if contains characters outside `[a-z_]`
    pub fn new(value: impl Into<String>) -> Result<Self, AadError> {
        let value = value.into();

        if value.is_empty() {
            return Err(AadError::EmptyFieldKey);
        }

        for ch in value.chars() {
            if !matches!(ch, 'a'..='z' | '_') {
                return Err(AadError::InvalidFieldKey {
                    key: value,
                    reason: format!("contains invalid character '{ch}', only [a-z_] allowed"),
                });
            }
        }

        Ok(Self(value))
    }

    /// Returns the validated key. Guaranteed non-empty, `[a-z_]` only.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True if the key matches any reserved field name (`v`, `tenant`, `resource`, `purpose`, `ts`).
    #[must_use]
    pub fn is_reserved(&self) -> bool {
        RESERVED_KEYS.contains(&self.0.as_str())
    }

    /// Validates that this key is a valid extension key (`x_<app>_<field>`).
    ///
    /// Pattern: `^x_[a-z]+_[a-z_]+$`
    /// - must start with `x_`
    /// - app segment: one or more lowercase letters
    /// - field segment: one or more lowercase letters or underscores
    ///
    /// # Errors
    ///
    /// Returns `InvalidExtensionKeyFormat` if the key doesn't match the pattern.
    pub fn validate_as_extension(&self) -> Result<(), AadError> {
        let s = &self.0;

        if !s.starts_with("x_") {
            return Err(self.ext_format_err());
        }

        let rest = &s[2..];
        let underscore_pos = rest.find('_');

        match underscore_pos {
            None | Some(0) => Err(self.ext_format_err()),
            Some(pos) => {
                let app = &rest[..pos];
                let field = &rest[pos + 1..];
                validate_app_segment(s, app)?;
                validate_field_segment(s, field)
            }
        }
    }

    fn ext_format_err(&self) -> AadError {
        AadError::InvalidExtensionKeyFormat {
            key: self.0.clone(),
            expected_pattern: EXTENSION_KEY_PATTERN,
        }
    }
}

fn validate_app_segment(key: &str, app: &str) -> Result<(), AadError> {
    if app.is_empty() || !app.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(AadError::InvalidExtensionKeyFormat {
            key: key.to_owned(),
            expected_pattern: EXTENSION_KEY_PATTERN,
        });
    }
    Ok(())
}

fn validate_field_segment(key: &str, field: &str) -> Result<(), AadError> {
    if field.is_empty() {
        return Err(AadError::InvalidExtensionKeyFormat {
            key: key.to_owned(),
            expected_pattern: EXTENSION_KEY_PATTERN,
        });
    }
    for ch in field.chars() {
        if !matches!(ch, 'a'..='z' | '_') {
            return Err(AadError::InvalidExtensionKeyFormat {
                key: key.to_owned(),
                expected_pattern: EXTENSION_KEY_PATTERN,
            });
        }
    }
    Ok(())
}

impl TryFrom<String> for FieldKey {
    type Error = AadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for FieldKey {
    type Error = AadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for FieldKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for FieldKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}
