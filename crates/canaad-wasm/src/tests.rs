use canaad_core::AadError;

use super::*;

#[test]
fn test_canonicalize() {
    let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
    let result = canonicalize(json);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    let canonical = String::from_utf8(bytes).unwrap();
    assert_eq!(
        canonical,
        r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
    );
}

#[test]
fn test_canonicalize_string_fn() {
    let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
    let result = canonicalize_string(json);
    assert!(result.is_ok());

    let canonical = result.unwrap();
    assert_eq!(
        canonical,
        r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
    );
}

#[test]
fn test_validate_valid() {
    let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
    assert!(validate(json));
}

#[test]
fn test_validate_invalid() {
    // Missing required field
    let json = r#"{"v":1,"tenant":"org_abc"}"#;
    assert!(!validate(json));
}

#[test]
fn test_hash_fn() {
    let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
    let result = hash(json);
    assert!(result.is_ok());

    let hash_bytes = result.unwrap();
    assert_eq!(hash_bytes.len(), 32);
}

#[test]
fn test_constants() {
    assert_eq!(spec_version(), 1);
    assert!((max_safe_integer() - 9_007_199_254_740_991.0).abs() < f64::EPSILON);
    assert_eq!(max_serialized_bytes(), 16 * 1024);
}

#[test]
fn test_builder_basic() {
    let builder = AadBuilder::new().tenant("org_abc").resource("secrets/db").purpose("encryption");

    let result = builder.build_string();
    assert!(result.is_ok());

    let canonical = result.unwrap();
    assert_eq!(
        canonical,
        r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
    );
}

#[test]
fn test_builder_with_timestamp() {
    let builder =
        AadBuilder::new().tenant("org").resource("res").purpose("test").timestamp(1_706_400_000.0);

    let result = builder.build_string();
    assert!(result.is_ok());

    let canonical = result.unwrap();
    assert!(canonical.contains(r#""ts":1706400000"#));
}

#[test]
fn test_builder_with_extensions() {
    let builder = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_string("x_vault_cluster", "us-east-1")
        .extension_int("x_app_priority", 5.0);

    let result = builder.build_string();
    assert!(result.is_ok());

    let canonical = result.unwrap();
    assert!(canonical.contains(r#""x_app_priority":5"#));
    assert!(canonical.contains(r#""x_vault_cluster":"us-east-1""#));
}

#[test]
fn test_builder_missing_required() {
    let builder = AadBuilder::new().tenant("org").resource("res");

    // Test internal build_context which doesn't require JsError
    let result = builder.build_context();
    assert!(result.is_err());
}

// f64 boundary validation tests

#[test]
fn test_timestamp_nan_fails() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(f64::NAN)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { ref field, reason: "NaN" }) if field == "timestamp"),
        "expected InvalidFloat(NaN) for timestamp, got: {result:?}"
    );
}

#[test]
fn test_timestamp_infinity_fails() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(f64::INFINITY)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { ref field, reason: "Infinity" }) if field == "timestamp"),
        "expected InvalidFloat(Infinity) for timestamp, got: {result:?}"
    );
}

#[test]
fn test_timestamp_negative_fails() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(-1.0)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { ref field, reason: "negative" }) if field == "timestamp"),
        "expected InvalidFloat(negative) for timestamp, got: {result:?}"
    );
}

#[test]
fn test_timestamp_fractional_fails() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(3.7)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { ref field, reason: "fractional" }) if field == "timestamp"),
        "expected InvalidFloat(fractional) for timestamp, got: {result:?}"
    );
}

#[test]
fn test_timestamp_negative_zero_succeeds() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(-0.0)
        .build_context();
    assert!(result.is_ok(), "expected Ok for -0.0, got: {result:?}");
}

#[test]
fn test_extension_int_nan_fails() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_int("x_app_v", f64::NAN)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { reason: "NaN", .. })),
        "expected InvalidFloat(NaN) for extension, got: {result:?}"
    );
}

#[test]
fn test_timestamp_exceeds_max_safe_integer() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(9_007_199_254_740_992.0)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { ref field, reason: "exceeds MAX_SAFE_INTEGER" }) if field == "timestamp"),
        "expected InvalidFloat(exceeds MAX_SAFE_INTEGER) for timestamp, got: {result:?}"
    );
}

#[test]
fn test_extension_int_exceeds_max_safe_integer() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_int("x_app_v", 9_007_199_254_740_992.0)
        .build_context();
    assert!(
        matches!(result, Err(AadError::InvalidFloat { reason: "exceeds MAX_SAFE_INTEGER", .. })),
        "expected InvalidFloat(exceeds MAX_SAFE_INTEGER) for extension, got: {result:?}"
    );
}

/// SOC-01: duplicate extension keys in `AadBuilder::build_context()` must return `DuplicateKey`.
#[test]
fn test_builder_duplicate_extension_key_errors() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_string("x_app_foo", "a")
        .extension_string("x_app_foo", "b")
        .build_context();

    match result {
        Err(AadError::DuplicateKey { key }) => {
            assert_eq!(key, "x_app_foo");
        }
        other => panic!("expected DuplicateKey {{ key: \"x_app_foo\" }}, got: {other:?}"),
    }
}

#[test]
fn test_valid_values_still_work() {
    let result = AadBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .timestamp(1_706_400_000.0)
        .extension_int("x_app_v", 5.0)
        .build_context();
    assert!(result.is_ok(), "expected Ok for valid values, got: {result:?}");
}
