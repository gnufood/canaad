//! Section 10 known-answer vectors from the AAD specification.

use crate::{canonicalize_string, AadContext};
use sha2::{Digest, Sha256};

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// =============================================================================
// Section 10.1: Minimal Required Fields
// =============================================================================

#[test]
fn test_vector_10_1_minimal_fields() {
    let input = r#"{
        "v": 1,
        "tenant": "org_abc",
        "resource": "secrets/db",
        "purpose": "encryption"
    }"#;

    let expected_canonical =
        r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;

    let expected_sha256 = "03fdc63d2f82815eb0a97e6f1a02890e152c021a795142b9c22e2b31a3bd83eb";

    let expected_hex = "7b22707572706f7365223a22656e6372797074696f6e222c227265736f75726365223a22736563726574732f6462222c2274656e616e74223a226f72675f616263222c2276223a317d";

    let canonical = canonicalize_string(input).expect("should parse and canonicalize");
    assert_eq!(canonical, expected_canonical);

    let utf8_hex = hex::encode(canonical.as_bytes());
    assert_eq!(utf8_hex, expected_hex);

    let sha256 = sha256_hex(canonical.as_bytes());
    assert_eq!(sha256, expected_sha256);
}

#[test]
fn test_vector_10_1_via_builder() {
    let ctx =
        AadContext::new("org_abc", "secrets/db", "encryption").expect("should create context");

    let canonical = ctx.canonicalize_string().expect("should canonicalize");
    let expected = r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;

    assert_eq!(canonical, expected);
}

// =============================================================================
// Section 10.2: All Fields Including Optional
// =============================================================================

#[test]
fn test_vector_10_2_all_fields() {
    let input = r#"{
        "v": 1,
        "tenant": "org_abc",
        "resource": "secrets/db/prod",
        "purpose": "encryption-at-rest",
        "ts": 1706400000
    }"#;

    let expected_canonical = r#"{"purpose":"encryption-at-rest","resource":"secrets/db/prod","tenant":"org_abc","ts":1706400000,"v":1}"#;

    let canonical = canonicalize_string(input).expect("should parse and canonicalize");
    assert_eq!(canonical, expected_canonical);
}

#[test]
fn test_vector_10_2_via_builder() {
    let ctx = AadContext::new("org_abc", "secrets/db/prod", "encryption-at-rest")
        .expect("should create context")
        .with_timestamp(1706400000)
        .expect("should add timestamp");

    let canonical = ctx.canonicalize_string().expect("should canonicalize");
    let expected = r#"{"purpose":"encryption-at-rest","resource":"secrets/db/prod","tenant":"org_abc","ts":1706400000,"v":1}"#;

    assert_eq!(canonical, expected);
}

// =============================================================================
// Section 10.3: Unicode in Values
// =============================================================================

#[test]
fn test_vector_10_3_unicode() {
    let input = r#"{
        "v": 1,
        "tenant": "组织_测试",
        "resource": "data/🔐/secret",
        "purpose": "encryption"
    }"#;

    let expected_canonical =
        r#"{"purpose":"encryption","resource":"data/🔐/secret","tenant":"组织_测试","v":1}"#;

    let canonical = canonicalize_string(input).expect("should parse and canonicalize");
    assert_eq!(canonical, expected_canonical);

    assert!(canonical.contains("组织_测试"));
    assert!(canonical.contains("🔐"));
}

#[test]
fn test_vector_10_3_via_builder() {
    let ctx = AadContext::new("组织_测试", "data/🔐/secret", "encryption")
        .expect("should create context");

    let canonical = ctx.canonicalize_string().expect("should canonicalize");

    assert!(canonical.contains("组织_测试"));
    assert!(canonical.contains("🔐"));
}

// =============================================================================
// Section 10.4: Extension Fields
// =============================================================================

#[test]
fn test_vector_10_4_extension_fields() {
    let input = r#"{
        "v": 1,
        "tenant": "org_abc",
        "resource": "vault/key",
        "purpose": "key-wrapping",
        "x_vault_cluster": "us-east-1"
    }"#;

    let expected_canonical = r#"{"purpose":"key-wrapping","resource":"vault/key","tenant":"org_abc","v":1,"x_vault_cluster":"us-east-1"}"#;

    let canonical = canonicalize_string(input).expect("should parse and canonicalize");
    assert_eq!(canonical, expected_canonical);
}

#[test]
fn test_vector_10_4_via_builder() {
    let ctx = AadContext::new("org_abc", "vault/key", "key-wrapping")
        .expect("should create context")
        .with_string_extension("x_vault_cluster", "us-east-1")
        .expect("should add extension");

    let canonical = ctx.canonicalize_string().expect("should canonicalize");
    let expected = r#"{"purpose":"key-wrapping","resource":"vault/key","tenant":"org_abc","v":1,"x_vault_cluster":"us-east-1"}"#;

    assert_eq!(canonical, expected);
}

// =============================================================================
// Section 10.5: JCS Edge Cases
// =============================================================================

#[test]
fn test_vector_10_5_jcs_edge_cases() {
    // Input uses \u000A for newline and has escaped quotes
    let input = r#"{
        "v": 1,
        "tenant": "org\u000Atest",
        "resource": "path/with\"quotes",
        "purpose": "test",
        "ts": 9007199254740991
    }"#;

    // Canonical output should use \n not \u000A, and preserve escaped quotes
    let expected_canonical = r#"{"purpose":"test","resource":"path/with\"quotes","tenant":"org\ntest","ts":9007199254740991,"v":1}"#;

    let canonical = canonicalize_string(input).expect("should parse and canonicalize");
    assert_eq!(canonical, expected_canonical);

    assert!(canonical.contains(r#"\n"#));
    assert!(canonical.contains(r#"\""#));
    assert!(canonical.contains("9007199254740991"));
}

#[test]
fn test_vector_10_5_max_safe_integer() {
    // Test the maximum safe integer value
    let ctx = AadContext::new("org", "res", "test")
        .expect("should create context")
        .with_timestamp(9007199254740991)
        .expect("should accept max safe integer");

    let canonical = ctx.canonicalize_string().expect("should canonicalize");
    assert!(canonical.contains("9007199254740991"));
}
