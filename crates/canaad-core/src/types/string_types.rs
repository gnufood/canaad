//! `Tenant`, `Resource`, `Purpose` newtypes with length and NUL-byte validation.

use crate::error::AadError;
use serde::{Deserialize, Serialize};
use std::fmt;

pub(super) fn check_nul_byte(s: &str, field: &'static str) -> Result<(), AadError> {
    if s.contains('\0') {
        return Err(AadError::NulByteInValue { field });
    }
    Ok(())
}

/// Tenant identifier (1-256 bytes, no NUL).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Tenant(String);

impl Tenant {
    /// Validates and wraps a tenant string.
    ///
    /// # Errors
    ///
    /// - `FieldTooShort` if empty
    /// - `FieldTooLong` if > 256 bytes
    /// - `NulByteInValue` if contains a NUL byte
    pub fn new(value: impl Into<String>) -> Result<Self, AadError> {
        let value = value.into();
        check_nul_byte(&value, "tenant")?;

        let len = value.len();
        if len == 0 {
            return Err(AadError::FieldTooShort { field: "tenant", min_bytes: 1, actual_bytes: 0 });
        }
        if len > 256 {
            return Err(AadError::FieldTooLong {
                field: "tenant",
                max_bytes: 256,
                actual_bytes: len,
            });
        }
        Ok(Self(value))
    }

    /// Returns the validated tenant. Guaranteed 1-256 bytes, no NUL.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Tenant {
    type Error = AadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Tenant> for String {
    fn from(val: Tenant) -> Self {
        val.0
    }
}

impl fmt::Display for Tenant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Resource path or identifier (1-1024 bytes, no NUL).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Resource(String);

impl Resource {
    /// Validates and wraps a resource string.
    ///
    /// # Errors
    ///
    /// - `FieldTooShort` if empty
    /// - `FieldTooLong` if > 1024 bytes
    /// - `NulByteInValue` if contains a NUL byte
    pub fn new(value: impl Into<String>) -> Result<Self, AadError> {
        let value = value.into();
        check_nul_byte(&value, "resource")?;

        let len = value.len();
        if len == 0 {
            return Err(AadError::FieldTooShort {
                field: "resource",
                min_bytes: 1,
                actual_bytes: 0,
            });
        }
        if len > 1024 {
            return Err(AadError::FieldTooLong {
                field: "resource",
                max_bytes: 1024,
                actual_bytes: len,
            });
        }
        Ok(Self(value))
    }

    /// Returns the validated resource. Guaranteed 1-1024 bytes, no NUL.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Resource {
    type Error = AadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Resource> for String {
    fn from(val: Resource) -> Self {
        val.0
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Purpose/usage context (1+ bytes, no NUL).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Purpose(String);

impl Purpose {
    /// Validates and wraps a purpose string.
    ///
    /// # Errors
    ///
    /// - `FieldTooShort` if empty
    /// - `NulByteInValue` if contains a NUL byte
    pub fn new(value: impl Into<String>) -> Result<Self, AadError> {
        let value = value.into();
        check_nul_byte(&value, "purpose")?;

        if value.is_empty() {
            return Err(AadError::FieldTooShort {
                field: "purpose",
                min_bytes: 1,
                actual_bytes: 0,
            });
        }
        Ok(Self(value))
    }

    /// Returns the validated purpose. Guaranteed non-empty, no NUL.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Purpose {
    type Error = AadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Purpose> for String {
    fn from(val: Purpose) -> Self {
        val.0
    }
}

impl fmt::Display for Purpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
