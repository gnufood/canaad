//! Free-standing public API: parse, validate, canonicalize, `canonicalize_string`.

use crate::context::AadContext;
use crate::error::AadError;
use crate::parse::parse_aad;

/// Parses a JSON string into a validated `AadContext`.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or violates AAD constraints.
pub fn parse(json: &str) -> Result<AadContext, AadError> {
    let parsed = parse_aad(json)?;
    AadContext::try_from(parsed)
}

/// Alias for `parse` — same validation, same return type.
///
/// The distinction is semantic: use `parse` when you need the returned
/// `AadContext`; use `validate` when you only care whether the input is valid
/// and want to make that intent explicit at the call site.
///
/// # Errors
///
/// Returns an error if the JSON is invalid or violates AAD constraints.
pub fn validate(json: &str) -> Result<AadContext, AadError> {
    parse(json)
}

/// Canonicalizes a JSON string to bytes.
///
/// Parses, validates, then returns the RFC 8785 canonical byte form.
///
/// # Errors
///
/// Returns an error if the JSON is invalid, violates AAD constraints,
/// or the canonical output exceeds 16 KiB.
pub fn canonicalize(json: &str) -> Result<Vec<u8>, AadError> {
    parse(json)?.canonicalize()
}

/// Canonicalizes a JSON string to a UTF-8 string.
///
/// Parses, validates, then returns the RFC 8785 canonical string form.
///
/// # Errors
///
/// Returns an error if the JSON is invalid, violates AAD constraints,
/// or the canonical output exceeds 16 KiB.
pub fn canonicalize_string(json: &str) -> Result<String, AadError> {
    parse(json)?.canonicalize_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_canonicalize() {
        let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
        let canonical = canonicalize_string(json).unwrap();

        assert_eq!(
            canonical,
            r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
        );
    }

    #[test]
    fn test_parse_roundtrip() {
        let original =
            r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;
        let ctx = parse(original).unwrap();
        let canonical = ctx.canonicalize_string().unwrap();

        assert_eq!(canonical, original);
    }

    #[test]
    fn test_validate_is_parse() {
        let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
        let via_parse = parse(json).unwrap();
        let via_validate = validate(json).unwrap();

        assert_eq!(via_parse, via_validate);
    }
}
