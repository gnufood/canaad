//! AAD field extractor: parses and validates AAD JSON objects.

use super::scan::parse_json_with_duplicate_check;
use crate::error::{AadError, JsonType};
use crate::types::{
    ExtensionValue, Extensions, FieldKey, Purpose, Resource, SafeInt, Tenant, RESERVED_KEYS,
};
use serde_json::{Map, Value};

/// Maximum serialized AAD size in bytes (16 KiB).
pub const MAX_AAD_SIZE: usize = 16 * 1024;

/// Current supported schema version.
pub const CURRENT_VERSION: u64 = 1;

/// Parsed AAD fields before full validation.
#[derive(Debug)]
pub(crate) struct ParsedAad {
    pub(crate) version: SafeInt,
    pub(crate) tenant: Tenant,
    pub(crate) resource: Resource,
    pub(crate) purpose: Purpose,
    pub(crate) timestamp: Option<SafeInt>,
    pub(crate) extensions: Extensions,
}

/// Applies core rules only: size check, duplicate-key detection, JSON parse, object assert.
///
/// Does not enforce any profile-specific fields (version, required keys, extensions).
/// Use `parse_aad` for the full default-profile validation.
///
/// # Errors
///
/// Returns an error if the input exceeds `MAX_AAD_SIZE`, contains duplicate keys,
/// is syntactically invalid JSON, or is not a JSON object.
pub(crate) fn parse_object(json: &str) -> Result<serde_json::Value, AadError> {
    if json.len() > MAX_AAD_SIZE {
        return Err(AadError::SerializedTooLarge {
            max_bytes: MAX_AAD_SIZE,
            actual_bytes: json.len(),
        });
    }

    let value = parse_json_with_duplicate_check(json)?;
    if !value.is_object() {
        return Err(AadError::InvalidJson { message: "input must be a JSON object".to_string() });
    }

    Ok(value)
}

/// Parses, duplicate-checks, and validates an AAD JSON string against the spec.
///
/// # Errors
///
/// Returns an error if the JSON is syntactically invalid, contains duplicate keys,
/// or violates any AAD specification constraint.
pub(crate) fn parse_aad(json: &str) -> Result<ParsedAad, AadError> {
    let value = parse_object(json)?;
    let obj = value.as_object().ok_or_else(|| AadError::InvalidJson {
        message: "AAD must be a JSON object".to_string(),
    })?;

    let version = extract_version(obj)?;
    validate_field_names(obj)?;

    let tenant = extract_string_field(obj, "tenant").and_then(Tenant::new)?;
    let resource = extract_string_field(obj, "resource").and_then(Resource::new)?;
    let purpose = extract_string_field(obj, "purpose").and_then(Purpose::new)?;
    let timestamp = extract_optional_timestamp(obj)?;
    let extensions = extract_extensions(obj)?;

    Ok(ParsedAad { version, tenant, resource, purpose, timestamp, extensions })
}

fn extract_version(obj: &Map<String, Value>) -> Result<SafeInt, AadError> {
    match obj.get("v") {
        None => Err(AadError::MissingRequiredField { field: "v" }),
        Some(v) => {
            let n = v.as_u64().ok_or_else(|| AadError::WrongFieldType {
                field: "v",
                expected: "integer",
                actual: JsonType::from(v),
            })?;
            if n != CURRENT_VERSION {
                return Err(AadError::UnsupportedVersion { version: n });
            }
            SafeInt::new(n)
        }
    }
}

fn validate_field_names(obj: &Map<String, Value>) -> Result<(), AadError> {
    for key in obj.keys() {
        if RESERVED_KEYS.contains(&key.as_str()) {
            continue;
        }
        let field_key = FieldKey::new(key.clone())?;
        if !key.starts_with("x_") {
            return Err(AadError::UnknownField { field: key.clone(), version: CURRENT_VERSION });
        }
        field_key.validate_as_extension()?;
    }
    Ok(())
}

fn extract_string_field(obj: &Map<String, Value>, field: &'static str) -> Result<String, AadError> {
    obj.get(field).map_or(Err(AadError::MissingRequiredField { field }), |v| {
        v.as_str().map(String::from).ok_or_else(|| AadError::WrongFieldType {
            field,
            expected: "string",
            actual: JsonType::from(v),
        })
    })
}

fn extract_optional_timestamp(obj: &Map<String, Value>) -> Result<Option<SafeInt>, AadError> {
    match obj.get("ts") {
        None => Ok(None),
        Some(v) => match v.as_u64() {
            Some(n) => Ok(Some(SafeInt::new(n)?)),
            None => v.as_i64().map_or_else(
                || {
                    Err(AadError::WrongFieldType {
                        field: "ts",
                        expected: "integer",
                        actual: JsonType::from(v),
                    })
                },
                |i| Err(AadError::NegativeInteger { value: i }),
            ),
        },
    }
}

fn extract_extensions(obj: &Map<String, Value>) -> Result<Extensions, AadError> {
    let mut extensions = Extensions::new();
    for (key, value) in obj {
        if key.starts_with("x_") {
            let field_key = FieldKey::new(key)?;
            field_key.validate_as_extension()?;
            let ext_value = parse_extension_value(value)?;
            extensions.insert(field_key, ext_value);
        }
    }
    Ok(extensions)
}

fn parse_extension_value(value: &Value) -> Result<ExtensionValue, AadError> {
    match value {
        Value::String(s) => ExtensionValue::string(s),
        Value::Number(n) => n.as_u64().map_or_else(
            || {
                n.as_i64().map_or(
                    Err(AadError::WrongFieldType {
                        field: "extension",
                        expected: "string or integer",
                        actual: JsonType::Number,
                    }),
                    |i| Err(AadError::NegativeInteger { value: i }),
                )
            },
            ExtensionValue::integer,
        ),
        _ => Err(AadError::WrongFieldType {
            field: "extension",
            expected: "string or integer",
            actual: JsonType::from(value),
        }),
    }
}
