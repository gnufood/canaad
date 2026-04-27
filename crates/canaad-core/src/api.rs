//! Free-standing public API: default-profile and generic-object functions.

use crate::canon::canonicalize_value;
use crate::context::AadContext;
use crate::error::AadError;
use crate::parse::{parse_aad, parse_object};

// ---------------------------------------------------------------------------
// Default-profile API (requires v, tenant, resource, purpose)
// ---------------------------------------------------------------------------

/// Parses a JSON string into a validated `AadContext`.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or violates AAD constraints.
pub fn parse_default(json: &str) -> Result<AadContext, AadError> {
    let parsed = parse_aad(json)?;
    AadContext::try_from(parsed)
}

/// Alias for `parse_default` — same validation, same return type.
///
/// The distinction is semantic: use `parse_default` when you need the returned
/// `AadContext`; use `validate_default` when you only care whether the input is
/// valid and want to make that intent explicit at the call site.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or violates AAD constraints.
pub fn validate_default(json: &str) -> Result<AadContext, AadError> {
    parse_default(json)
}

/// Canonicalizes a JSON string to bytes using the default AAD profile.
///
/// Parses, validates, then returns the RFC 8785 canonical byte form.
///
/// # Errors
///
/// Returns an error if the JSON is invalid, violates AAD constraints,
/// or the canonical output exceeds 16 KiB.
pub fn canonicalize_default(json: &str) -> Result<Vec<u8>, AadError> {
    parse_default(json)?.canonicalize()
}

/// Canonicalizes a JSON string to a UTF-8 string using the default AAD profile.
///
/// Parses, validates, then returns the RFC 8785 canonical string form.
///
/// # Errors
///
/// Returns an error if the JSON is invalid, violates AAD constraints,
/// or the canonical output exceeds 16 KiB.
pub fn canonicalize_default_string(json: &str) -> Result<String, AadError> {
    parse_default(json)?.canonicalize_string()
}

// ---------------------------------------------------------------------------
// Generic-object API (core rules only — no required fields)
// ---------------------------------------------------------------------------

/// Canonicalizes any valid JSON object to bytes per RFC 8785.
///
/// Applies core rules only: size check, duplicate-key detection, object assertion.
/// Does not enforce any default-profile fields (`v`, `tenant`, `resource`, `purpose`).
///
/// # Errors
///
/// Returns an error if the input exceeds 16 KiB, contains duplicate keys,
/// is not valid JSON, is not a JSON object, or the canonical output exceeds 16 KiB.
pub fn canonicalize_object(json: &str) -> Result<Vec<u8>, AadError> {
    let value = parse_object(json)?;
    canonicalize_value(&value)
}

/// Canonicalizes any valid JSON object to a UTF-8 string per RFC 8785.
///
/// Applies core rules only: size check, duplicate-key detection, object assertion.
/// Does not enforce any default-profile fields (`v`, `tenant`, `resource`, `purpose`).
///
/// # Errors
///
/// Returns an error if the input exceeds 16 KiB, contains duplicate keys,
/// is not valid JSON, is not a JSON object, or the canonical output exceeds 16 KiB.
pub fn canonicalize_object_string(json: &str) -> Result<String, AadError> {
    let bytes = canonicalize_object(json)?;
    String::from_utf8(bytes).map_err(|e| AadError::InvalidJson {
        message: format!("canonicalized output is not valid UTF-8: {e}"),
    })
}

/// Validates any JSON string against core rules (size, duplicate keys, object shape).
///
/// Does not enforce any default-profile fields (`v`, `tenant`, `resource`, `purpose`).
///
/// # Errors
///
/// Returns an error if the input exceeds 16 KiB, contains duplicate keys,
/// is not valid JSON, or is not a JSON object.
pub fn validate_object(json: &str) -> Result<(), AadError> {
    parse_object(json).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_canonicalize_default() {
        let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
        let canonical = canonicalize_default_string(json).unwrap();

        assert_eq!(
            canonical,
            r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
        );
    }

    #[test]
    fn test_parse_default_roundtrip() {
        let original =
            r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;
        let ctx = parse_default(original).unwrap();
        let canonical = ctx.canonicalize_string().unwrap();

        assert_eq!(canonical, original);
    }

    #[test]
    fn test_validate_default_is_parse_default() {
        let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
        let via_parse = parse_default(json).unwrap();
        let via_validate = validate_default(json).unwrap();

        assert_eq!(via_parse, via_validate);
    }

    #[test]
    fn test_canonicalize_object_sorts_keys() {
        let json = r#"{"z":1,"a":2,"m":3}"#;
        let canonical = canonicalize_object_string(json).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_canonicalize_object_no_profile_fields_required() {
        // A plain object with no AAD-profile fields should succeed.
        let json = r#"{"foo":"bar","baz":42}"#;
        let result = canonicalize_object(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_object_rejects_array() {
        let json = r#"[1,2,3]"#;
        let result = validate_object(json);
        assert!(matches!(result, Err(AadError::InvalidJson { .. })));
    }

    #[test]
    fn test_validate_object_rejects_duplicate_keys() {
        let json = r#"{"a":1,"a":2}"#;
        let result = validate_object(json);
        assert!(matches!(result, Err(AadError::DuplicateKey { .. })));
    }
}
