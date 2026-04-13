//! Boundary and edge case tests.

use crate::{canonicalize_string, parse, AadError};

#[test]
fn test_extension_field_with_integer_value() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_count":42}"#;
    let result = parse(input);
    assert!(result.is_ok());

    let ctx = result.unwrap();
    assert_eq!(ctx.extensions().len(), 1);
}

#[test]
fn test_multiple_extension_fields() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":"test","x_app_field":"value","x_other_key":"data"}"#;
    let result = parse(input);
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
    let result = parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_tenant_at_max_length() {
    let tenant = "a".repeat(256);
    let input = format!(r#"{{"v":1,"tenant":"{}","resource":"res","purpose":"test"}}"#, tenant);
    let result = parse(&input);
    assert!(result.is_ok());
}

#[test]
fn test_tenant_exceeds_max_length() {
    let tenant = "a".repeat(257);
    let input = format!(r#"{{"v":1,"tenant":"{}","resource":"res","purpose":"test"}}"#, tenant);
    let result = parse(&input);
    assert!(matches!(result, Err(AadError::FieldTooLong { field: "tenant", .. })));
}

#[test]
fn test_resource_at_max_length() {
    let resource = "a".repeat(1024);
    let input = format!(r#"{{"v":1,"tenant":"org","resource":"{}","purpose":"test"}}"#, resource);
    let result = parse(&input);
    assert!(result.is_ok());
}

#[test]
fn test_resource_exceeds_max_length() {
    let resource = "a".repeat(1025);
    let input = format!(r#"{{"v":1,"tenant":"org","resource":"{}","purpose":"test"}}"#, resource);
    let result = parse(&input);
    assert!(matches!(result, Err(AadError::FieldTooLong { field: "resource", .. })));
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

    let canonical = canonicalize_string(input).unwrap();

    // Canonical output should have no whitespace
    assert!(!canonical.contains(' '));
    assert!(!canonical.contains('\n'));
    assert_eq!(canonical, r#"{"purpose":"test","resource":"res","tenant":"org","v":1}"#);
}

#[test]
fn test_special_characters_in_strings() {
    let input = r#"{"v":1,"tenant":"org/abc","resource":"path\\to\\file","purpose":"test\ttab"}"#;
    let result = parse(input);
    assert!(result.is_ok());

    let ctx = result.unwrap();
    assert_eq!(ctx.tenant(), "org/abc");
    assert_eq!(ctx.resource(), "path\\to\\file");
    assert_eq!(ctx.purpose(), "test\ttab");
}

#[test]
fn test_empty_resource() {
    let input = r#"{"v":1,"tenant":"org","resource":"","purpose":"test"}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::FieldTooShort { field: "resource", .. })));
}

#[test]
fn test_empty_purpose() {
    let input = r#"{"v":1,"tenant":"org","resource":"res","purpose":""}"#;
    let result = parse(input);
    assert!(matches!(result, Err(AadError::FieldTooShort { field: "purpose", .. })));
}
