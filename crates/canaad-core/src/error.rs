//! Error types for AAD parsing and validation.

use std::fmt;
use thiserror::Error;

/// All possible errors during AAD parsing, validation, and canonicalization.
#[derive(Debug, Error)]
pub enum AadError {
    /// Integer value exceeds the safe range (0 to 2^53-1).
    #[error("field '{field}' integer out of range: {value} exceeds maximum safe value {max}")]
    IntegerOutOfRange {
        /// The field whose value was out of range
        field: String,
        /// The value that was out of range
        value: u64,
        /// The maximum allowed value (2^53-1)
        max: u64,
    },

    /// Negative integer encountered where unsigned was expected.
    #[error("field '{field}' negative integer not allowed: {value}")]
    NegativeInteger {
        /// The field that received the negative value
        field: String,
        /// The negative value encountered
        value: i64,
    },

    /// Field key is empty.
    #[error("field key cannot be empty")]
    EmptyFieldKey,

    /// Field key contains invalid characters.
    #[error("invalid field key '{key}': {reason}")]
    InvalidFieldKey {
        /// The invalid key
        key: String,
        /// Explanation of why it's invalid
        reason: String,
    },

    /// Attempted to use a reserved key name as an extension.
    #[error("reserved key '{key}' cannot be used as extension field")]
    ReservedKeyAsExtension {
        /// The reserved key that was misused
        key: String,
    },

    /// Extension key does not match required pattern `x_<app>_<field>`.
    #[error("invalid extension key format '{key}': expected pattern {expected_pattern}")]
    InvalidExtensionKeyFormat {
        /// The invalid extension key
        key: String,
        /// The expected pattern description
        expected_pattern: &'static str,
    },

    /// String field is shorter than minimum length.
    #[error("field '{field}' too short: minimum {min_bytes} bytes, got {actual_bytes}")]
    FieldTooShort {
        /// The field name
        field: &'static str,
        /// Minimum required bytes
        min_bytes: usize,
        /// Actual bytes provided
        actual_bytes: usize,
    },

    /// String field exceeds maximum length.
    #[error("field '{field}' too long: maximum {max_bytes} bytes, got {actual_bytes}")]
    FieldTooLong {
        /// The field name
        field: &'static str,
        /// Maximum allowed bytes
        max_bytes: usize,
        /// Actual bytes provided
        actual_bytes: usize,
    },

    /// NUL byte (0x00) found in string value.
    #[error("field '{field}' contains NUL byte (0x00)")]
    NulByteInValue {
        /// The field containing the NUL byte
        field: &'static str,
    },

    /// Required field is missing from the AAD object.
    #[error("missing required field: {field}")]
    MissingRequiredField {
        /// The name of the missing field
        field: &'static str,
    },

    /// Duplicate key found in JSON object.
    #[error("duplicate key: '{key}'")]
    DuplicateKey {
        /// The duplicated key
        key: String,
    },

    /// Unknown field for the specified schema version.
    #[error("unknown field '{field}' for schema version {version}")]
    UnknownField {
        /// The unknown field name
        field: String,
        /// The schema version
        version: u64,
    },

    /// Unsupported schema version.
    #[error("unsupported schema version: {version}")]
    UnsupportedVersion {
        /// The unsupported version number
        version: u64,
    },

    /// Field has wrong type (e.g., string instead of integer).
    #[error("field '{field}' has wrong type: expected {expected}, got {actual}")]
    WrongFieldType {
        /// The field with wrong type
        field: &'static str,
        /// The expected type
        expected: &'static str,
        /// The actual type found
        actual: JsonType,
    },

    /// Serialized AAD exceeds the 16 KiB limit.
    #[error("serialized AAD too large: maximum {max_bytes} bytes, got {actual_bytes}")]
    SerializedTooLarge {
        /// Maximum allowed bytes (16384)
        max_bytes: usize,
        /// Actual serialized size
        actual_bytes: usize,
    },

    /// Invalid JSON syntax or structure.
    #[error("invalid JSON: {message}")]
    InvalidJson {
        /// Description of the JSON error
        message: String,
    },

    /// Non-integer or non-finite float where an integer was required.
    #[error("field '{field}' is not a valid integer: {reason}")]
    InvalidFloat {
        /// The field name
        field: String,
        /// `"NaN"`, `"Infinity"`, `"negative"`, `"fractional"`, or `"exceeds MAX_SAFE_INTEGER"`
        reason: &'static str,
    },
}

/// JSON value type for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonType {
    /// JSON null value
    Null,
    /// JSON boolean value
    Bool,
    /// JSON number value
    Number,
    /// JSON string value
    String,
    /// JSON array value
    Array,
    /// JSON object value
    Object,
}

impl fmt::Display for JsonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool => write!(f, "boolean"),
            Self::Number => write!(f, "number"),
            Self::String => write!(f, "string"),
            Self::Array => write!(f, "array"),
            Self::Object => write!(f, "object"),
        }
    }
}

impl From<&serde_json::Value> for JsonType {
    fn from(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(_) => Self::Bool,
            serde_json::Value::Number(_) => Self::Number,
            serde_json::Value::String(_) => Self::String,
            serde_json::Value::Array(_) => Self::Array,
            serde_json::Value::Object(_) => Self::Object,
        }
    }
}
