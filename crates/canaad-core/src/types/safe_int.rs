//! `SafeInt` newtype: integer values bounded to the JS-safe range.
use crate::error::AadError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Maximum safe integer value (2^53 - 1) for cross-platform compatibility.
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// A safe integer in the range 0 to 2^53-1.
///
/// Ensures integer values are compatible with JavaScript's Number type
/// and all common runtime environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafeInt(u64);

impl SafeInt {
    /// Validates `value` against the 2^53-1 bound.
    ///
    /// # Errors
    ///
    /// Returns `IntegerOutOfRange` if `value` exceeds `MAX_SAFE_INTEGER`.
    pub fn new_for_field(value: u64, field: impl Into<String>) -> Result<Self, AadError> {
        if value > MAX_SAFE_INTEGER {
            return Err(AadError::IntegerOutOfRange {
                field: field.into(),
                value,
                max: MAX_SAFE_INTEGER,
            });
        }
        Ok(Self(value))
    }

    /// Validates `value` against the 2^53-1 bound. Errors carry `field: "unknown"` —
    /// prefer `new_for_field` when the caller knows the field name.
    ///
    /// # Errors
    ///
    /// Returns `IntegerOutOfRange` if `value` exceeds `MAX_SAFE_INTEGER`.
    pub fn new(value: u64) -> Result<Self, AadError> {
        Self::new_for_field(value, "unknown")
    }

    /// Returns the validated integer. Guaranteed in `[0, MAX_SAFE_INTEGER]` (2^53-1).
    #[inline]
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for SafeInt {
    type Error = AadError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<i64> for SafeInt {
    type Error = AadError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(AadError::NegativeInteger { field: "unknown".to_string(), value });
        }
        Self::new(value.unsigned_abs())
    }
}

impl TryFrom<usize> for SafeInt {
    type Error = AadError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        // usize is at most 64 bits on all supported targets; conversion is infallible
        Self::new(u64::try_from(value).unwrap_or_else(|_| unreachable!()))
    }
}

impl From<SafeInt> for u64 {
    fn from(val: SafeInt) -> Self {
        val.0
    }
}

impl fmt::Display for SafeInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SafeInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for SafeInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}
