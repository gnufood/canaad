//! JSON parsing with duplicate key detection.
//!
//! The standard `serde_json` library silently drops duplicate keys,
//! keeping only the last value. This module implements custom parsing
//! that detects and rejects duplicate keys per the AAD specification.

mod aad;
mod scan;

pub(crate) use aad::parse_aad;
pub(crate) use aad::ParsedAad;
pub use aad::{CURRENT_VERSION, MAX_AAD_SIZE};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AadError;
    use scan::parse_json_with_duplicate_check;

    #[test]
    fn test_parse_valid_minimal() {
        let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
        let result = parse_aad(json);
        assert!(result.is_ok());

        let aad = result.unwrap();
        assert_eq!(aad.version.value(), 1);
        assert_eq!(aad.tenant.as_str(), "org_abc");
        assert_eq!(aad.resource.as_str(), "secrets/db");
        assert_eq!(aad.purpose.as_str(), "encryption");
        assert!(aad.timestamp.is_none());
        assert!(aad.extensions.is_empty());
    }

    #[test]
    fn test_parse_with_timestamp() {
        let json = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":1706400000}"#;
        let result = parse_aad(json);
        assert!(result.is_ok());

        let aad = result.unwrap();
        assert_eq!(aad.timestamp.unwrap().value(), 1_706_400_000);
    }

    #[test]
    fn test_parse_with_extension() {
        let json = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_vault_cluster":"us-east-1"}"#;
        let result = parse_aad(json);
        assert!(result.is_ok());

        let aad = result.unwrap();
        assert_eq!(aad.extensions.len(), 1);
    }

    #[test]
    fn test_duplicate_key_detection() {
        let json = r#"{"v":1,"tenant":"org","tenant":"other","resource":"res","purpose":"test"}"#;
        let result = parse_aad(json);
        assert!(matches!(result, Err(AadError::DuplicateKey { key }) if key == "tenant"));
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"v":1,"tenant":"org","resource":"res"}"#;
        let result = parse_aad(json);
        assert!(matches!(result, Err(AadError::MissingRequiredField { field: "purpose" })));
    }

    #[test]
    fn test_unknown_field() {
        let json = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","unknown":"value"}"#;
        let result = parse_aad(json);
        assert!(
            matches!(result, Err(AadError::UnknownField { field, version: 1 }) if field == "unknown")
        );
    }

    #[test]
    fn test_invalid_extension_key() {
        let json = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_foo":"value"}"#;
        let result = parse_aad(json);
        assert!(matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })));
    }

    #[test]
    fn test_unsupported_version() {
        let json = r#"{"v":2,"tenant":"org","resource":"res","purpose":"test"}"#;
        let result = parse_aad(json);
        assert!(matches!(result, Err(AadError::UnsupportedVersion { version: 2 })));
    }

    #[test]
    fn test_wrong_field_type() {
        let json = r#"{"v":"1","tenant":"org","resource":"res","purpose":"test"}"#;
        let result = parse_aad(json);
        assert!(matches!(
            result,
            Err(AadError::WrongFieldType { field: "v", expected: "integer", .. })
        ));
    }

    #[test]
    fn test_integer_out_of_range() {
        let json =
            r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":9007199254740992}"#;
        let result = parse_aad(json);
        assert!(matches!(result, Err(AadError::IntegerOutOfRange { .. })));
    }

    #[test]
    fn test_size_limit() {
        let big_resource = "x".repeat(MAX_AAD_SIZE + 1);
        let json =
            format!(r#"{{"v":1,"tenant":"org","resource":"{}","purpose":"test"}}"#, big_resource);
        let result = parse_aad(&json);
        assert!(matches!(result, Err(AadError::SerializedTooLarge { .. })));
    }

    #[test]
    fn test_unicode_escapes() {
        let json = r#"{"v":1,"tenant":"test\u0041","resource":"res","purpose":"test"}"#;
        let result = parse_aad(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_duplicate_not_relevant() {
        let json = r#"{"v":1}"#;
        let result = parse_json_with_duplicate_check(json);
        assert!(result.is_ok());
    }
}
