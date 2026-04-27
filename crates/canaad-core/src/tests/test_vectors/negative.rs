//! Rejection and error-case tests — inputs that must fail validation.

use crate::{canonicalize_object, parse_default, validate_object, AadError};
use rstest::rstest;

// =============================================================================
// Integer range errors (default profile)
// =============================================================================

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":9007199254740992}"#,
    "ts one above MAX_SAFE_INTEGER"
)]
fn test_negative_integer_above_max_safe(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::IntegerOutOfRange { .. })));
}

#[rstest]
#[case(r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":-1}"#, "ts negative")]
fn test_negative_integer_value(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::NegativeInteger { .. })));
}

// =============================================================================
// Empty / too-short string fields (default profile)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"tenant":"","resource":"res","purpose":"test"}"#, "tenant")]
fn test_negative_empty_tenant(#[case] input: &str, #[case] field: &'static str) {
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::FieldTooShort { field: f, .. }) if f == field));
}

// =============================================================================
// NUL byte in string fields (default profile)
// =============================================================================

#[rstest]
#[case("tenant")]
fn test_negative_nul_byte_in_tenant(#[case] _field: &str) {
    // NUL byte encoded as \u0000 JSON escape — raw string keeps the escape as literal text.
    let input = r#"{"v":1,"tenant":"org\u0000abc","resource":"res","purpose":"test"}"#;
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::NulByteInValue { field: "tenant" })));
}

// =============================================================================
// Unknown / unrecognised fields (default profile)
// =============================================================================

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","unknown":"value"}"#,
    "non-extension unknown field"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","unknown":"value"}"#,
    "unknown field non-extension"
)]
fn test_negative_unknown_field(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::UnknownField { .. })));
}

// =============================================================================
// Missing required fields (default profile)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"tenant":"org","resource":"res"}"#, "purpose")]
fn test_negative_missing_required_field(#[case] input: &str, #[case] field: &'static str) {
    let result = parse_default(input);
    assert!(matches!(result, Err(AadError::MissingRequiredField { field: f }) if f == field));
}

// =============================================================================
// Duplicate key — generic detection
// =============================================================================

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","tenant":"other","resource":"res","purpose":"test"}"#,
    "duplicate tenant"
)]
fn test_negative_duplicate_key(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
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
    match parse_default(input) {
        Err(AadError::DuplicateKey { key }) => {
            assert_eq!(key, "x_a_b", "expected duplicate key 'x_a_b', got: {key:?}");
        }
        other => panic!("expected DuplicateKey, got: {other:?}"),
    }
}

// =============================================================================
// Duplicate key — exact key extracted for each position (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"v":2,"tenant":"org","resource":"res","purpose":"test"}"#, "v", "first field")]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","resource":"other","purpose":"test"}"#,
    "resource",
    "middle field"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","purpose":"other"}"#,
    "purpose",
    "last field"
)]
fn test_negative_duplicate_key_position(
    #[case] input: &str,
    #[case] expected_key: &str,
    #[case] _position: &str,
) {
    match parse_default(input) {
        Err(AadError::DuplicateKey { ref key }) => {
            assert_eq!(
                key, expected_key,
                "expected duplicate key '{expected_key}' ({_position}), got {key:?}"
            );
        }
        other => panic!("expected DuplicateKey ({_position}), got: {other:?}"),
    }
}

// =============================================================================
// Invalid extension key formats (default profile, parameterized)
// =============================================================================

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_foo":"value"}"#,
    "x_foo missing second segment"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x__field":"value"}"#,
    "x__field empty app segment"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_":"value"}"#,
    "x_app_ empty field part"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_foo":"bad"}"#,
    "x_foo no second segment (duplicate label variant)"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x__bar":"bad"}"#,
    "x__bar empty app segment (duplicate label variant)"
)]
fn test_negative_invalid_extension_key_format(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })),
        "{_label}: must return InvalidExtensionKeyFormat; got {result:?}"
    );
}

/// `x_` — starts with `x_` but has nothing after the prefix.
#[test]
fn test_negative_extension_key_x_bare() {
    // `x_` alone: validate_as_extension fires after FieldKey::new succeeds (it's valid [a-z_]).
    let result = validate_object(r#"{"v":1,"x_":"bad"}"#);
    // parse_object doesn't run profile validation, but validate_object just checks object shape.
    // Use parse_default to trigger full validation.
    let result2 =
        parse_default(r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_":"bad"}"#);
    assert!(
        matches!(result2, Err(AadError::InvalidExtensionKeyFormat { .. })),
        "bare x_ key must return InvalidExtensionKeyFormat; got {result2:?}"
    );
    // validate_object (generic) accepts any key — just confirming it passes object shape.
    let _ = result;
}

// =============================================================================
// Unsupported version numbers (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":2,"tenant":"org","resource":"res","purpose":"test"}"#, 2_u64)]
#[case(r#"{"v":0,"tenant":"org","resource":"res","purpose":"test"}"#, 0_u64)]
fn test_negative_unsupported_version(#[case] input: &str, #[case] version: u64) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::UnsupportedVersion { version: v }) if v == version),
        "v={version} must return UnsupportedVersion; got {result:?}"
    );
}

// =============================================================================
// Wrong field types (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":"1","tenant":"org","resource":"res","purpose":"test"}"#, "v", "string version")]
#[case(r#"{"v":1,"tenant":123,"resource":"res","purpose":"test"}"#, "tenant", "integer tenant")]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":"1706400000"}"#,
    "ts",
    "string ts"
)]
fn test_negative_wrong_field_type(
    #[case] input: &str,
    #[case] field: &'static str,
    #[case] _label: &str,
) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::WrongFieldType { field: f, .. }) if f == field),
        "{_label}: must return WrongFieldType for field '{field}'; got {result:?}"
    );
}

// =============================================================================
// Invalid JSON / non-object at default-profile level (parameterized)
// =============================================================================

#[rstest]
// Missing closing brace
#[case(r#"{"v":1,"tenant":"org","resource":"res","purpose":"test""#, "truncated JSON")]
// Array instead of object
#[case(r#"["v", 1, "tenant", "org"]"#, "JSON array")]
fn test_negative_invalid_json(#[case] input: &str, #[case] _label: &str) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::InvalidJson { .. })),
        "{_label}: must return InvalidJson; got {result:?}"
    );
}

// =============================================================================
// SEC-03: `validate_field_names` must propagate the precise `InvalidFieldKey` reason
// =============================================================================

/// SEC-03: `validate_field_names` must propagate the precise `InvalidFieldKey` reason from
/// `FieldKey::new` rather than replacing it with a generic string.  The reason must identify
/// the specific offending character.
#[test]
fn test_invalid_field_key_reason_contains_character() {
    // "X_app_foo" fails FieldKey::new because 'X' is not in [a-z_].
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","X_app_foo":"val"}"#;
    let result = parse_default(input);
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

// =============================================================================
// Additional non-object inputs (generic layer, parameterized)
// =============================================================================

#[rstest]
#[case(r#""hello""#, "JSON string")]
#[case("42", "JSON number")]
#[case("true", "JSON boolean true")]
#[case("false", "JSON boolean false")]
#[case("null", "JSON null")]
fn test_negative_generic_rejects_non_object(#[case] input: &str, #[case] label: &str) {
    let result = canonicalize_object(input);
    assert!(
        matches!(result, Err(AadError::InvalidJson { .. })),
        "{label} must return InvalidJson; got {result:?}"
    );
}
