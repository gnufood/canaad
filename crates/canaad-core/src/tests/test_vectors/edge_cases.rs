//! Boundary and edge case tests.

use crate::{
    canonicalize_default_string, canonicalize_object, canonicalize_object_string, parse_default,
    AadError, MAX_AAD_SIZE, MAX_SAFE_INTEGER,
};
use rstest::rstest;

// =============================================================================
// Generic-layer edge cases — inputs that canonicalize successfully
// =============================================================================

#[test]
fn test_generic_empty_object() {
    // Empty object should succeed — no required fields in generic layer.
    let result = canonicalize_object_string("{}");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "{}");
}

#[test]
fn test_generic_single_string_field() {
    let result = canonicalize_object_string(r#"{"hello":"world"}"#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"{"hello":"world"}"#);
}

#[test]
fn test_generic_single_integer_zero() {
    let result = canonicalize_object_string(r#"{"n":0}"#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#"{"n":0}"#);
}

#[test]
fn test_generic_single_integer_max_safe() {
    let input = format!(r#"{{"n":{MAX_SAFE_INTEGER}}}"#);
    let result = canonicalize_object_string(&input);
    assert!(result.is_ok());
    assert!(result.unwrap().contains(&MAX_SAFE_INTEGER.to_string()));
}

#[test]
fn test_generic_integer_above_max_safe_succeeds() {
    // The generic layer does NOT validate integer bounds — that is profile-level.
    // One above MAX_SAFE_INTEGER must succeed at the generic layer.
    let above = MAX_SAFE_INTEGER + 1;
    let input = format!(r#"{{"n":{above}}}"#);
    let result = canonicalize_object(input.as_str());
    assert!(
        result.is_ok(),
        "generic layer must not reject integers above MAX_SAFE_INTEGER; got {result:?}"
    );
}

#[test]
fn test_generic_unicode_values() {
    // Emoji, CJK, and RTL as values — not keys (keys must be valid JSON strings, which they are,
    // but the spec says keys are sorted lexicographically by Unicode code unit order).
    let input = r#"{"emoji":"🔐","cjk":"组织","rtl":"مرحبا"}"#;
    let result = canonicalize_object_string(input);
    assert!(result.is_ok(), "unicode values must be accepted: {result:?}");
    let canonical = result.unwrap();
    // All values should be present verbatim.
    assert!(canonical.contains("🔐"));
    assert!(canonical.contains("组织"));
    assert!(canonical.contains("مرحبا"));
}

#[test]
fn test_generic_twenty_plus_fields_sorted() {
    // Build an object with 26 fields in reverse alphabetical order.
    let reversed_keys: Vec<String> =
        (b'a'..=b'z').rev().map(|c| format!(r#""{}":{},"#, char::from(c), c)).collect();
    let fields = reversed_keys.join("");
    let fields = fields.trim_end_matches(',');
    let input = format!("{{{fields}}}");

    let canonical = canonicalize_object_string(&input).unwrap();

    // Extract key order from canonical output.
    let value: serde_json::Value = serde_json::from_str(&canonical).unwrap();
    let keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "canonical output must have keys in lexicographic order");
}

#[test]
fn test_generic_leading_trailing_whitespace() {
    let input = "   {\"a\":1}   ";
    let result = canonicalize_object_string(input);
    assert!(result.is_ok(), "leading/trailing whitespace must be handled: {result:?}");
    assert_eq!(result.unwrap(), r#"{"a":1}"#);
}

#[test]
fn test_generic_exactly_max_aad_size() {
    // Build an input that is exactly MAX_AAD_SIZE bytes.
    // We need a JSON object whose raw input length equals MAX_AAD_SIZE.
    // Construct `{"a":"<padding>"}` where padding fills to the target.
    let prefix = r#"{"a":""#;
    let suffix = r#""}"#;
    let padding_len = MAX_AAD_SIZE - prefix.len() - suffix.len();
    let input = format!("{prefix}{}{suffix}", "x".repeat(padding_len));
    assert_eq!(input.len(), MAX_AAD_SIZE, "test setup: input must be exactly MAX_AAD_SIZE");

    let result = canonicalize_object(&input);
    // The canonical output for a single-field string object will be smaller than
    // the input (no escaping needed for 'x'), so this should succeed.
    assert!(result.is_ok(), "input at exactly MAX_AAD_SIZE should succeed: {result:?}");
}

#[test]
fn test_generic_exceeds_max_aad_size() {
    // Build an input that is MAX_AAD_SIZE + 1 bytes.
    let prefix = r#"{"a":""#;
    let suffix = r#""}"#;
    let padding_len = MAX_AAD_SIZE + 1 - prefix.len() - suffix.len();
    let input = format!("{prefix}{}{suffix}", "x".repeat(padding_len));
    assert_eq!(input.len(), MAX_AAD_SIZE + 1, "test setup: input must be MAX_AAD_SIZE+1");

    let result = canonicalize_object(&input);
    assert!(
        matches!(result, Err(AadError::SerializedTooLarge { .. })),
        "input exceeding MAX_AAD_SIZE must return SerializedTooLarge; got {result:?}"
    );
}

// =============================================================================
// Generic-layer — non-object JSON types must be rejected (parameterized)
// =============================================================================

#[rstest]
#[case("[1,2,3]", "JSON array")]
#[case(r#""hello""#, "JSON string")]
#[case("null", "JSON null")]
#[case("42", "JSON number")]
#[case("", "empty string")]
fn test_generic_rejects_non_object(#[case] input: &str, #[case] label: &str) {
    let result = canonicalize_object(input);
    assert!(
        matches!(result, Err(AadError::InvalidJson { .. })),
        "{label} must return InvalidJson; got {result:?}"
    );
}

// =============================================================================
// Generic-layer — duplicate key detection (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"a":1,"a":1}"#, "a", "identical values")]
#[case(r#"{"a":1,"b":2,"a":3}"#, "a", "last-position duplicate")]
fn test_generic_duplicate_key(
    #[case] input: &str,
    #[case] expected_key: &str,
    #[case] label: &str,
) {
    let result = canonicalize_object(input);
    assert!(
        matches!(result, Err(AadError::DuplicateKey { ref key }) if key == expected_key),
        "{label}: expected DuplicateKey for '{expected_key}'; got {result:?}"
    );
}

// =============================================================================
// Default-profile layer — extension key parsing
// =============================================================================

#[test]
fn test_extension_field_with_integer_value() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_count":42}"#;
    let result = parse_default(input);
    assert!(result.is_ok());

    let ctx = result.unwrap();
    assert_eq!(ctx.extensions().len(), 1);
}

#[test]
fn test_multiple_extension_fields() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_field":"value","x_other_key":"data"}"#;
    let result = parse_default(input);
    assert!(result.is_ok());

    let ctx = result.unwrap();
    assert_eq!(ctx.extensions().len(), 2);

    // Verify canonical ordering (extensions are sorted)
    let canonical = ctx.canonicalize_string().unwrap();
    let app_pos = canonical.find("x_app_field").unwrap();
    let other_pos = canonical.find("x_other_key").unwrap();
    assert!(app_pos < other_pos);
}

#[test]
fn test_extension_key_with_multiple_underscores_in_field() {
    // x_app_field_name is valid (field can contain underscores)
    let input =
        r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_field_name":"value"}"#;
    let result = parse_default(input);
    assert!(result.is_ok());
}

#[test]
fn test_whitespace_in_input_is_handled() {
    // Input with lots of whitespace
    let input = r#"
    {
        "v" : 1 ,
        "tenant" : "org" ,
        "resource" : "res" ,
        "purpose" : "test"
    }
    "#;

    let canonical = canonicalize_default_string(input).unwrap();

    // Canonical output should have no whitespace
    assert!(!canonical.contains(' '));
    assert!(!canonical.contains('\n'));
    assert_eq!(canonical, r#"{"purpose":"test","resource":"res","tenant":"org","v":1}"#);
}

#[test]
fn test_special_characters_in_strings() {
    let input = r#"{"v":1,"tenant":"org/abc","resource":"path\\to\\file","purpose":"test\ttab"}"#;
    let result = parse_default(input);
    assert!(result.is_ok());

    let ctx = result.unwrap();
    assert_eq!(ctx.tenant(), "org/abc");
    assert_eq!(ctx.resource(), "path\\to\\file");
    assert_eq!(ctx.purpose(), "test\ttab");
}

// =============================================================================
// Default-profile — field length boundaries (parameterized)
// =============================================================================

/// Tests that fields accept values at their exact maximum length and reject
/// values one byte beyond it.
#[rstest]
// tenant: min=1, max=256
#[case("tenant", "a", "ok", true)]
#[case("tenant", "a".repeat(256), "ok", true)]
#[case("tenant", "a".repeat(257), "FieldTooLong", false)]
// resource: min=1, max=1024
#[case("resource", "a", "ok", true)]
#[case("resource", "a".repeat(1024), "ok", true)]
#[case("resource", "a".repeat(1025), "FieldTooLong", false)]
fn test_default_field_length_boundary(
    #[case] field: &str,
    #[case] value: String,
    #[case] _label: &str,
    #[case] should_succeed: bool,
) {
    let input = match field {
        "tenant" => format!(r#"{{"v":1,"tenant":"{value}","resource":"res","purpose":"test"}}"#),
        "resource" => format!(r#"{{"v":1,"tenant":"org","resource":"{value}","purpose":"test"}}"#),
        _ => unreachable!(),
    };
    let result = parse_default(&input);
    if should_succeed {
        assert!(result.is_ok(), "field '{field}' at '{_label}' must succeed; got {result:?}");
    } else {
        let field_static: &'static str = match field {
            "tenant" => "tenant",
            "resource" => "resource",
            _ => unreachable!(),
        };
        assert!(
            matches!(result, Err(AadError::FieldTooLong { field: f, .. }) if f == field_static),
            "field '{field}' at '{_label}' must return FieldTooLong; got {result:?}"
        );
    }
}

// =============================================================================
// Default-profile — empty / too-short string fields (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"tenant":"org","resource":"","purpose":"test"}"#, "resource")]
#[case(r#"{"v":1,"tenant":"org","resource":"res","purpose":""}"#, "purpose")]
fn test_default_empty_field(#[case] input: &str, #[case] field: &'static str) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::FieldTooShort { field: f, .. }) if f == field),
        "empty '{field}' must return FieldTooShort; got {result:?}"
    );
}

// =============================================================================
// Default-profile — single-byte minimum valid field values (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"tenant":"x","resource":"res","purpose":"test"}"#, "tenant at 1 byte")]
#[case(r#"{"v":1,"tenant":"org","resource":"r","purpose":"test"}"#, "resource at 1 byte")]
#[case(r#"{"v":1,"tenant":"org","resource":"res","purpose":"p"}"#, "purpose at 1 byte")]
fn test_default_field_single_byte_valid(#[case] input: &str, #[case] label: &str) {
    assert!(parse_default(input).is_ok(), "{label} must succeed");
}

// =============================================================================
// Default-profile — NUL byte in string fields (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":1,"tenant":"org\u0000abc","resource":"res","purpose":"test"}"#, "tenant")]
#[case(r#"{"v":1,"tenant":"org","resource":"res\u0000path","purpose":"test"}"#, "resource")]
#[case(r#"{"v":1,"tenant":"org","resource":"res","purpose":"enc\u0000ryption"}"#, "purpose")]
fn test_default_nul_byte_in_field(#[case] input: &str, #[case] field: &'static str) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::NulByteInValue { field: f }) if f == field),
        "NUL in '{field}' must return NulByteInValue; got {result:?}"
    );
}

// =============================================================================
// Default-profile — `ts` (timestamp) field boundaries (parameterized)
// =============================================================================

#[test]
fn test_default_ts_exactly_zero() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":0}"#;
    let ctx = parse_default(input).expect("ts=0 is valid");
    assert_eq!(ctx.timestamp(), Some(0));
}

#[test]
fn test_default_ts_exactly_max_safe_integer() {
    let input = format!(
        r#"{{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":{MAX_SAFE_INTEGER}}}"#
    );
    let ctx = parse_default(&input).expect("ts=MAX_SAFE_INTEGER is valid");
    assert_eq!(ctx.timestamp(), Some(MAX_SAFE_INTEGER));
}

#[rstest]
#[case(MAX_SAFE_INTEGER + 1, "Err(IntegerOutOfRange)", true)]
fn test_default_ts_above_max_safe_integer(
    #[case] above: u64,
    #[case] _label: &str,
    #[case] _expect_err: bool,
) {
    let input =
        format!(r#"{{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":{above}}}"#);
    assert!(
        matches!(parse_default(&input), Err(AadError::IntegerOutOfRange { .. })),
        "ts above MAX_SAFE_INTEGER must return IntegerOutOfRange"
    );
}

#[test]
fn test_default_ts_negative() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":-1}"#;
    assert!(
        matches!(parse_default(input), Err(AadError::NegativeInteger { .. })),
        "negative ts must return NegativeInteger"
    );
}

#[test]
fn test_default_ts_as_string() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","ts":"1706400000"}"#;
    assert!(
        matches!(parse_default(input), Err(AadError::WrongFieldType { field: "ts", .. })),
        "string ts must return WrongFieldType"
    );
}

// =============================================================================
// Default-profile — extension key format validation (parameterized)
// =============================================================================

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_a_b":"ok"}"#,
    "minimum valid x_a_b"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_a_b":0}"#,
    "extension integer 0"
)]
fn test_default_extension_key_valid(#[case] input: &str, #[case] label: &str) {
    assert!(parse_default(input).is_ok(), "{label} must succeed");
}

#[rstest]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_foo":"bad"}"#,
    "x_foo missing second segment"
)]
#[case(
    r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x__foo":"bad"}"#,
    "x__foo empty app segment"
)]
fn test_default_extension_key_invalid_format(#[case] input: &str, #[case] label: &str) {
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::InvalidExtensionKeyFormat { .. })),
        "{label} must return InvalidExtensionKeyFormat; got {result:?}"
    );
}

#[test]
fn test_default_extension_key_uppercase_in_app_segment() {
    // x_FOO_bar has uppercase in app segment — FieldKey::new will reject 'F' first,
    // producing InvalidFieldKey rather than InvalidExtensionKeyFormat.
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_FOO_bar":"bad"}"#;
    let result = parse_default(input);
    assert!(
        matches!(result, Err(AadError::InvalidFieldKey { .. })),
        "uppercase in app segment must return InvalidFieldKey; got {result:?}"
    );
}

// =============================================================================
// Default-profile — extension value integer boundaries (parameterized)
// =============================================================================

#[rstest]
#[case(0_u64, true, "extension integer 0")]
#[case(MAX_SAFE_INTEGER, true, "extension integer MAX_SAFE_INTEGER")]
fn test_default_extension_value_integer_valid(
    #[case] value: u64,
    #[case] _should_ok: bool,
    #[case] label: &str,
) {
    let input =
        format!(r#"{{"v":1,"tenant":"org","resource":"res","purpose":"test","x_a_b":{value}}}"#);
    assert!(parse_default(&input).is_ok(), "{label} must succeed");
}

#[test]
fn test_default_extension_value_integer_above_max_safe() {
    let above = MAX_SAFE_INTEGER + 1;
    let input =
        format!(r#"{{"v":1,"tenant":"org","resource":"res","purpose":"test","x_a_b":{above}}}"#);
    assert!(
        matches!(parse_default(&input), Err(AadError::IntegerOutOfRange { .. })),
        "extension integer above MAX_SAFE_INTEGER must return IntegerOutOfRange"
    );
}

// =============================================================================
// Default-profile — version field validation (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"v":2,"tenant":"org","resource":"res","purpose":"test"}"#, 2_u64)]
#[case(r#"{"v":0,"tenant":"org","resource":"res","purpose":"test"}"#, 0_u64)]
fn test_default_version_rejected(#[case] input: &str, #[case] version: u64) {
    assert!(
        matches!(parse_default(input), Err(AadError::UnsupportedVersion { version: v }) if v == version),
        "v={version} must return UnsupportedVersion"
    );
}

#[test]
fn test_default_version_string_rejected() {
    let input = r#"{"v":"1","tenant":"org","resource":"res","purpose":"test"}"#;
    assert!(
        matches!(parse_default(input), Err(AadError::WrongFieldType { field: "v", .. })),
        "string v must return WrongFieldType"
    );
}

// =============================================================================
// Default-profile — missing required fields (parameterized)
// =============================================================================

#[rstest]
#[case(r#"{"tenant":"org","resource":"res","purpose":"test"}"#, "v")]
#[case(r#"{"v":1,"resource":"res","purpose":"test"}"#, "tenant")]
#[case(r#"{"v":1,"tenant":"org","purpose":"test"}"#, "resource")]
#[case(r#"{"v":1,"tenant":"org","resource":"res"}"#, "purpose")]
fn test_default_missing_required_field(#[case] input: &str, #[case] field: &'static str) {
    assert!(
        matches!(parse_default(input), Err(AadError::MissingRequiredField { field: f }) if f == field),
        "missing '{field}' must return MissingRequiredField"
    );
}
