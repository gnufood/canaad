//! Rejection and error-case tests — inputs that must fail validation.

use crate::{parse, AadError};

#[test]
fn test_negative_integer_above_max_safe() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":9007199254740992}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::IntegerOutOfRange { .. })));
}

#[test]
fn test_negative_integer_value() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":-1}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::NegativeInteger { .. })));
}

#[test]
fn test_negative_empty_tenant() {
    let input = r#"{"v":1,"tenant":"","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::FieldTooShort { field: "tenant", .. })));
}

#[test]
fn test_negative_nul_byte_in_tenant() {
    // NUL byte encoded as \u0000
    let input = r#"{"v":1,"tenant":"org\u0000abc","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::NulByteInValue { field: "tenant" })));
}

#[test]
fn test_negative_unknown_field() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","unknown":"value"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::UnknownField { .. })));
}

#[test]
fn test_negative_missing_required_field() {
    let input = r#"{"v":1,"tenant":"org","resource":"res"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::MissingRequiredField { field: "purpose" })));
}

#[test]
fn test_negative_duplicate_key() {
    let input = r#"{"v":1,"tenant":"org","tenant":"other","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::DuplicateKey { .. })));
}

/// SEC-02: The duplicate key must be extracted via thread-local, not by parsing
/// serde_json's error message text (which could change across versions).
/// This asserts the exact key — if the implementation still relied on
/// the old string-stripping approach, a change in serde_json's error format
/// would yield a wrong or empty key.
#[test]
fn test_duplicate_key_exact_key_extracted() {
    // parse() goes through parse_json_with_duplicate_check internally.
    // The wrapping object structure doesn't matter; duplicate key detection
    // fires before field validation.
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_a_b":1,"x_a_b":2}"#;
    match parse(input) {
        Err(AadError::DuplicateKey { key }) => {
            assert_eq!(key, "x_a_b", "expected duplicate key 'x_a_b', got: {key:?}");
        }
        other => panic!("expected DuplicateKey, got: {other:?}"),
    }
}

#[test]
fn test_negative_invalid_extension_key_single_underscore() {
    // x_foo is missing the second underscore
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_foo":"value"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })));
}

#[test]
fn test_negative_invalid_extension_key_empty_app() {
    // x__field has empty app part
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x__field":"value"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })));
}

#[test]
fn test_negative_invalid_extension_key_empty_field() {
    // x_app_ has empty field part
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_":"value"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })));
}

#[test]
fn test_negative_unsupported_version() {
    let input = r#"{"v":2,"tenant":"org","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::UnsupportedVersion { version: 2 })));
}

#[test]
fn test_negative_version_zero() {
    let input = r#"{"v":0,"tenant":"org","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::UnsupportedVersion { version: 0 })));
}

#[test]
fn test_negative_wrong_field_type_version() {
    let input = r#"{"v":"1","tenant":"org","resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::WrongFieldType { field: "v", .. })));
}

#[test]
fn test_negative_wrong_field_type_tenant() {
    let input = r#"{"v":1,"tenant":123,"resource":"res","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::WrongFieldType { field: "tenant", .. })));
}

#[test]
fn test_negative_wrong_field_type_ts() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":"1706400000"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::WrongFieldType { field: "ts", .. })));
}

#[test]
fn test_negative_invalid_json() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test""#; // Missing closing brace
    let result = parse(input);
    assert!(matches!(result, Err(AadError::InvalidJson { .. })));
}

#[test]
fn test_negative_not_an_object() {
    let input = r#"["v", 1, "tenant", "org"]"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::InvalidJson { .. })));
}

/// SEC-03: `validate_field_names` must propagate the precise `InvalidFieldKey` reason from
/// `FieldKey::new` rather than replacing it with a generic string.  The reason must identify
/// the specific offending character.
#[test]
fn test_invalid_field_key_reason_contains_character() {
    // "X_app_foo" fails FieldKey::new because 'X' is not in [a-z_].
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","X_app_foo":"val"}"#;
    let result = parse(input);
    match result {
        Err(AadError::InvalidFieldKey { reason, .. }) => {
            assert!(
                reason.contains("'X'"),
                "expected reason to identify character 'X', got: {reason}"
            );
        }
        other => panic!("expected InvalidFieldKey, got: {other:?}"),
    }
}
