//! Unit tests for `AadContext` and `AadContextBuilder`.

use crate::context::builder::AadContextBuilder;
use crate::context::AadContext;
use crate::error::AadError;

#[test]
fn test_context_new() {
    let ctx = AadContext::new("org_abc", "secrets/db", "encryption").unwrap();

    assert_eq!(ctx.version(), 1);
    assert_eq!(ctx.tenant(), "org_abc");
    assert_eq!(ctx.resource(), "secrets/db");
    assert_eq!(ctx.purpose(), "encryption");
    assert!(ctx.timestamp().is_none());
    assert!(ctx.extensions().is_empty());
}

#[test]
fn test_context_with_timestamp() {
    let ctx = AadContext::new("org", "res", "test").unwrap().with_timestamp(1_706_400_000).unwrap();

    assert_eq!(ctx.timestamp(), Some(1_706_400_000));
}

#[test]
fn test_context_with_extension() {
    let ctx = AadContext::new("org", "res", "test")
        .unwrap()
        .with_string_extension("x_vault_cluster", "us-east-1")
        .unwrap();

    assert_eq!(ctx.extensions().len(), 1);
}

#[test]
fn test_builder() {
    let ctx = AadContextBuilder::new()
        .tenant("org_abc")
        .resource("secrets/db")
        .purpose("encryption")
        .timestamp(1_706_400_000)
        .extension_string("x_app_field", "value")
        .build()
        .unwrap();

    assert_eq!(ctx.tenant(), "org_abc");
    assert_eq!(ctx.timestamp(), Some(1_706_400_000));
    assert_eq!(ctx.extensions().len(), 1);
}

#[test]
fn test_builder_missing_required() {
    let result = AadContextBuilder::new().tenant("org").resource("res").build();

    assert!(matches!(result, Err(AadError::MissingRequiredField { field: "purpose" })));
}

#[test]
fn test_canonicalize_order() {
    let ctx = AadContext::new("org_abc", "secrets/db", "encryption").unwrap();
    let canonical = ctx.canonicalize_string().unwrap();

    assert_eq!(
        canonical,
        r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#
    );
}

#[test]
fn test_builder_extension_string_nul_byte_surfaces_error() {
    let result = AadContextBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_string("x_app_key", "val\0ue")
        .build();

    assert!(matches!(result, Err(AadError::NulByteInValue { .. })));
}

#[test]
fn test_builder_extension_int_out_of_range_surfaces_error() {
    let result = AadContextBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_int("x_app_key", u64::MAX)
        .build();

    assert!(matches!(result, Err(AadError::IntegerOutOfRange { .. })));
}

/// Asserts that canonicalized output keys are in strict lexicographic order.
///
/// Guards the manual `Serialize` impl against silent field-order drift
/// if new fields are added at the wrong position in future.
#[test]
fn test_serialize_key_order() {
    let ctx = AadContext::new("org_abc", "secrets/db", "encryption")
        .unwrap()
        .with_timestamp(1_706_400_000)
        .unwrap()
        .with_string_extension("x_app_env", "prod")
        .unwrap()
        .with_int_extension("x_app_ver", 42)
        .unwrap();

    let bytes = ctx.canonicalize().unwrap();

    // serde_json preserves insertion order in Map, so key iteration order
    // reflects the serialization order emitted by the Serialize impl.
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let emitted_keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();

    let mut sorted = emitted_keys.clone();
    sorted.sort_unstable();

    assert_eq!(
        emitted_keys, sorted,
        "canonical output keys must be in lexicographic order; got {emitted_keys:?}"
    );
}

/// SOC-01: duplicate extension keys in `AadContextBuilder::build()` must return `DuplicateKey`.
#[test]
fn test_builder_duplicate_extension_key_errors() {
    let result = AadContextBuilder::new()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extension_string("x_app_foo", "a")
        .extension_string("x_app_foo", "b")
        .build();

    match result {
        Err(AadError::DuplicateKey { key }) => {
            assert_eq!(key, "x_app_foo");
        }
        other => panic!("expected DuplicateKey {{ key: \"x_app_foo\" }}, got: {other:?}"),
    }
}

/// All five reserved keys must produce `ReservedKeyAsExtension`, not `InvalidExtensionKeyFormat`.
///
/// Guards D1-M7: `with_extension` previously skipped `is_reserved()` and let
/// `validate_as_extension` fire first, returning the wrong variant.
#[test]
fn test_with_extension_reserved_keys_return_correct_error() {
    let reserved = ["v", "tenant", "resource", "purpose", "ts"];

    for key in reserved {
        let result =
            AadContext::new("org", "res", "test").unwrap().with_string_extension(key, "value");

        assert!(
            matches!(result, Err(AadError::ReservedKeyAsExtension { .. })),
            "expected ReservedKeyAsExtension for key '{key}', got {result:?}"
        );
    }
}
