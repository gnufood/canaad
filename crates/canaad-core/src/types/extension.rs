//! `ExtensionValue` enum and `Extensions` type alias.

use crate::error::AadError;
use crate::types::{FieldKey, SafeInt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;

/// Extension field value: string or integer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionValue {
    /// String value (no NUL bytes allowed)
    String(String),
    /// Integer value in safe range
    Integer(SafeInt),
}

impl ExtensionValue {
    /// Validates and wraps a string extension value.
    ///
    /// # Errors
    ///
    /// Returns `NulByteInValue` if the string contains a NUL byte.
    pub fn string(value: impl Into<String>) -> Result<Self, AadError> {
        let value = value.into();
        if value.contains('\0') {
            return Err(AadError::NulByteInValue { field: "extension" });
        }
        Ok(Self::String(value))
    }

    /// Validates and wraps an integer extension value.
    ///
    /// # Errors
    ///
    /// Returns `IntegerOutOfRange` if `value` exceeds 2^53-1.
    pub fn integer(value: u64) -> Result<Self, AadError> {
        Ok(Self::Integer(SafeInt::new_for_field(value, "extension")?))
    }

    // `integer()` variant that names the specific field on `IntegerOutOfRange`
    pub(crate) fn integer_for_field(value: u64, field: &str) -> Result<Self, AadError> {
        Ok(Self::Integer(SafeInt::new_for_field(value, field)?))
    }
}

impl Serialize for ExtensionValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String(s) => serializer.serialize_str(s),
            Self::Integer(i) => i.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ExtensionValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};

        struct ExtensionValueVisitor;

        impl Visitor<'_> for ExtensionValueVisitor {
            type Value = ExtensionValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string or integer")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                ExtensionValue::string(v).map_err(Error::custom)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                ExtensionValue::string(v).map_err(Error::custom)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                ExtensionValue::integer(v).map_err(Error::custom)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v < 0 {
                    return Err(Error::custom(AadError::NegativeInteger {
                        field: "extension".to_string(),
                        value: v,
                    }));
                }
                ExtensionValue::integer(v.unsigned_abs()).map_err(Error::custom)
            }
        }

        deserializer.deserialize_any(ExtensionValueVisitor)
    }
}

/// Extension fields map.
pub type Extensions = BTreeMap<FieldKey, ExtensionValue>;
