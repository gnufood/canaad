//! Property-based tests using `proptest`.
//!
//! Covers determinism, key ordering, and round-trip properties for both
//! the generic-object layer and the default-profile layer.

use crate::{
    canonicalize_default, canonicalize_default_string, canonicalize_object,
    canonicalize_object_string, parse_default, MAX_SAFE_INTEGER,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generates strings of 1–256 bytes with no NUL bytes (valid tenant).
fn tenant_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[^\x00]{1,256}").unwrap()
}

/// Generates strings of 1–1024 bytes with no NUL bytes (valid resource).
fn resource_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[^\x00]{1,1024}").unwrap()
}

/// Generates strings of 1–128 bytes with no NUL bytes (valid purpose).
fn purpose_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[^\x00]{1,128}").unwrap()
}

/// Generates integers in [0, MAX_SAFE_INTEGER].
fn safe_int_strategy() -> impl Strategy<Value = u64> {
    0_u64..=MAX_SAFE_INTEGER
}

/// Builds a minimal default-profile JSON string from the three required fields.
fn build_json(tenant: &str, resource: &str, purpose: &str) -> String {
    // Use serde_json for correct JSON string escaping.
    let t = serde_json::to_string(tenant).unwrap();
    let r = serde_json::to_string(resource).unwrap();
    let p = serde_json::to_string(purpose).unwrap();
    format!(r#"{{"v":1,"tenant":{t},"resource":{r},"purpose":{p}}}"#)
}

// ---------------------------------------------------------------------------
// Generic-object layer: determinism
// ---------------------------------------------------------------------------

proptest! {
    /// Canonicalizing the same valid JSON object twice yields identical bytes.
    #[test]
    fn prop_generic_canonicalize_deterministic(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
        purpose in purpose_strategy(),
    ) {
        // Build a plain JSON object (no profile validation, just any object).
        let t = serde_json::to_string(&tenant).unwrap();
        let r = serde_json::to_string(&resource).unwrap();
        let p = serde_json::to_string(&purpose).unwrap();
        let json = format!(r#"{{"a":{t},"b":{r},"c":{p}}}"#);

        let first = canonicalize_object(&json).unwrap();
        let second = canonicalize_object(&json).unwrap();
        prop_assert_eq!(first, second);
    }

    /// `canonicalize_object` and `canonicalize_object_string` are consistent:
    /// bytes == string.into_bytes().
    #[test]
    fn prop_generic_bytes_string_consistency(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
    ) {
        let t = serde_json::to_string(&tenant).unwrap();
        let r = serde_json::to_string(&resource).unwrap();
        let json = format!(r#"{{"x":{t},"y":{r}}}"#);

        if let (Ok(bytes), Ok(s)) =
            (canonicalize_object(&json), canonicalize_object_string(&json))
        {
            prop_assert_eq!(bytes, s.into_bytes());
        }
    }
}

// ---------------------------------------------------------------------------
// Generic-object layer: key ordering
// ---------------------------------------------------------------------------

proptest! {
    /// For any two distinct field names, the canonical output always has the
    /// lexicographically smaller key first.
    ///
    /// Keys are generated as `keya`..`keyz` to guarantee they are valid,
    /// distinct, unambiguous JSON keys with no quoting complications.
    #[test]
    fn prop_generic_key_ordering(
        idx_a in 0_usize..26,
        idx_b in 0_usize..26,
        val_a in 0_u32..1000_u32,
        val_b in 0_u32..1000_u32,
    ) {
        prop_assume!(idx_a != idx_b);

        let key_a = format!("key{}", char::from(b'a' + idx_a as u8));
        let key_b = format!("key{}", char::from(b'a' + idx_b as u8));

        // Intentionally put key_a first in the raw JSON — canonical must sort them.
        let json = format!(r#"{{"{key_a}":{val_a},"{key_b}":{val_b}}}"#);
        let canonical = canonicalize_object_string(&json).unwrap();

        let pos_a = canonical.find(key_a.as_str()).expect("key_a in output");
        let pos_b = canonical.find(key_b.as_str()).expect("key_b in output");

        if key_a < key_b {
            prop_assert!(pos_a < pos_b, "lex-smaller '{key_a}' must precede '{key_b}'");
        } else {
            prop_assert!(pos_b < pos_a, "lex-smaller '{key_b}' must precede '{key_a}'");
        }
    }
}

// ---------------------------------------------------------------------------
// Default-profile layer: determinism
// ---------------------------------------------------------------------------

proptest! {
    /// Canonicalizing the same valid default-profile JSON twice yields identical bytes.
    #[test]
    fn prop_default_canonicalize_deterministic(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
        purpose in purpose_strategy(),
    ) {
        let json = build_json(&tenant, &resource, &purpose);
        match (canonicalize_default(&json), canonicalize_default(&json)) {
            (Ok(first), Ok(second)) => prop_assert_eq!(first, second),
            (Err(_), Err(_)) => {
                // Both failed identically — consistent, nothing to assert.
            }
            _ => prop_assert!(false, "non-deterministic: first and second calls disagree"),
        }
    }

    /// `canonicalize_default` and `canonicalize_default_string` produce the same data.
    #[test]
    fn prop_default_bytes_string_consistency(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
        purpose in purpose_strategy(),
    ) {
        let json = build_json(&tenant, &resource, &purpose);
        match (canonicalize_default(&json), canonicalize_default_string(&json)) {
            (Ok(bytes), Ok(s)) => prop_assert_eq!(bytes, s.into_bytes()),
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "bytes and string variants disagree on success/failure"),
        }
    }
}

// ---------------------------------------------------------------------------
// Default-profile layer: round-trip
// ---------------------------------------------------------------------------

proptest! {
    /// `parse_default` then `.canonicalize()` produces the same bytes as
    /// `canonicalize_default` on the same input.
    #[test]
    fn prop_default_roundtrip(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
        purpose in purpose_strategy(),
    ) {
        let json = build_json(&tenant, &resource, &purpose);
        if let Ok(ctx) = parse_default(&json) {
            let via_parse = ctx.canonicalize().expect("canonicalize after valid parse must succeed");
            let via_direct =
                canonicalize_default(&json).expect("direct must succeed when parse succeeded");
            prop_assert_eq!(via_parse, via_direct);
        }
    }

    /// Round-trip with optional timestamp field included.
    #[test]
    fn prop_default_roundtrip_with_timestamp(
        tenant in tenant_strategy(),
        resource in resource_strategy(),
        purpose in purpose_strategy(),
        ts in safe_int_strategy(),
    ) {
        let t = serde_json::to_string(&tenant).unwrap();
        let r = serde_json::to_string(&resource).unwrap();
        let p = serde_json::to_string(&purpose).unwrap();
        let json = format!(r#"{{"v":1,"tenant":{t},"resource":{r},"purpose":{p},"ts":{ts}}}"#);

        if let Ok(ctx) = parse_default(&json) {
            let via_parse = ctx.canonicalize().expect("canonicalize after valid parse must succeed");
            let via_direct =
                canonicalize_default(&json).expect("direct must succeed when parse succeeded");
            prop_assert_eq!(via_parse, via_direct);
        }
    }
}
