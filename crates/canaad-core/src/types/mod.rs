//! Core types for AAD fields with validation.

mod extension;
mod field_key;
mod safe_int;
mod string_types;

pub use extension::{ExtensionValue, Extensions};
pub use field_key::{FieldKey, RESERVED_KEYS};
pub use safe_int::{SafeInt, MAX_SAFE_INTEGER};
pub use string_types::{Purpose, Resource, Tenant};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AadError;
    use rstest::rstest;

    // -------------------------------------------------------------------------
    // SafeInt — valid boundary values (parameterized)
    // -------------------------------------------------------------------------

    #[rstest]
    #[case(0_u64, "zero")]
    #[case(1_u64, "one")]
    #[case(9_007_199_254_740_991_u64, "MAX_SAFE_INTEGER")]
    fn test_safe_int_valid(#[case] value: u64, #[case] label: &str) {
        let result = SafeInt::new(value);
        assert!(result.is_ok(), "SafeInt::new({value}) [{label}] must succeed");
        assert_eq!(result.unwrap().value(), value);
    }

    // -------------------------------------------------------------------------
    // SafeInt — overflow (parameterized)
    // -------------------------------------------------------------------------

    #[rstest]
    #[case(9_007_199_254_740_992_u64, "MAX_SAFE_INTEGER + 1")]
    #[case(u64::MAX, "u64::MAX")]
    fn test_safe_int_overflow(#[case] value: u64, #[case] label: &str) {
        let result = SafeInt::new(value);
        assert!(
            matches!(result, Err(AadError::IntegerOutOfRange { .. })),
            "SafeInt::new({value}) [{label}] must return IntegerOutOfRange; got {result:?}"
        );
    }

    #[test]
    fn test_safe_int_negative() {
        let result = SafeInt::try_from(-1i64);
        assert!(matches!(result, Err(AadError::NegativeInteger { .. })));
    }

    // -------------------------------------------------------------------------
    // SafeInt — try_from<u64> boundary values (parameterized)
    // -------------------------------------------------------------------------

    #[rstest]
    #[case(0_u64, true, "zero valid")]
    #[case(9_007_199_254_740_991_u64, true, "MAX_SAFE_INTEGER valid")]
    #[case(9_007_199_254_740_992_u64, false, "MAX_SAFE_INTEGER+1 rejected")]
    #[case(u64::MAX, false, "u64::MAX rejected")]
    fn test_safe_int_try_from_u64(
        #[case] value: u64,
        #[case] should_succeed: bool,
        #[case] label: &str,
    ) {
        let result = SafeInt::try_from(value);
        if should_succeed {
            assert!(result.is_ok(), "{label}: try_from({value}) must succeed; got {result:?}");
            assert_eq!(result.unwrap().value(), value, "{label}: value round-trips");
        } else {
            assert!(
                matches!(result, Err(AadError::IntegerOutOfRange { .. })),
                "{label}: try_from({value}) must return IntegerOutOfRange; got {result:?}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // String type constructors
    // -------------------------------------------------------------------------

    #[test]
    fn test_tenant_valid() {
        assert!(Tenant::new("org_abc").is_ok());
        assert!(Tenant::new("a").is_ok());
        assert!(Tenant::new("a".repeat(256)).is_ok());
    }

    #[test]
    fn test_tenant_empty() {
        let result = Tenant::new("");
        assert!(matches!(result, Err(AadError::FieldTooShort { field: "tenant", .. })));
    }

    #[test]
    fn test_tenant_too_long() {
        let result = Tenant::new("a".repeat(257));
        assert!(matches!(result, Err(AadError::FieldTooLong { field: "tenant", .. })));
    }

    #[test]
    fn test_tenant_nul_byte() {
        let result = Tenant::new("org\0abc");
        assert!(matches!(result, Err(AadError::NulByteInValue { field: "tenant" })));
    }

    #[test]
    fn test_resource_valid() {
        assert!(Resource::new("secrets/db").is_ok());
        assert!(Resource::new("a".repeat(1024)).is_ok());
    }

    #[test]
    fn test_resource_too_long() {
        let result = Resource::new("a".repeat(1025));
        assert!(matches!(result, Err(AadError::FieldTooLong { field: "resource", .. })));
    }

    #[test]
    fn test_purpose_valid() {
        assert!(Purpose::new("encryption").is_ok());
        assert!(Purpose::new("a").is_ok());
    }

    #[test]
    fn test_purpose_empty() {
        let result = Purpose::new("");
        assert!(matches!(result, Err(AadError::FieldTooShort { field: "purpose", .. })));
    }

    // -------------------------------------------------------------------------
    // FieldKey validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_field_key_valid() {
        assert!(FieldKey::new("v").is_ok());
        assert!(FieldKey::new("tenant").is_ok());
        assert!(FieldKey::new("x_vault_cluster").is_ok());
        assert!(FieldKey::new("a_b_c").is_ok());
    }

    #[test]
    fn test_field_key_empty() {
        let result = FieldKey::new("");
        assert!(matches!(result, Err(AadError::EmptyFieldKey)));
    }

    #[rstest]
    #[case("UPPER", "uppercase letters")]
    #[case("with-dash", "hyphen")]
    #[case("with123", "digits")]
    fn test_field_key_invalid_chars(#[case] key: &str, #[case] _reason: &str) {
        assert!(
            matches!(FieldKey::new(key), Err(AadError::InvalidFieldKey { .. })),
            "'{key}' ({_reason}) must return InvalidFieldKey"
        );
    }

    // -------------------------------------------------------------------------
    // Extension key validation (parameterized)
    // -------------------------------------------------------------------------

    #[rstest]
    #[case("x_vault_cluster", "two-segment with underscores")]
    #[case("x_myapp_region", "two-segment region")]
    #[case("x_app_field_name", "multi-segment field")]
    fn test_extension_key_valid(#[case] raw_key: &str, #[case] _label: &str) {
        let key = FieldKey::new(raw_key).expect("key must be a valid FieldKey");
        assert!(
            key.validate_as_extension().is_ok(),
            "'{raw_key}' ({_label}) must be a valid extension key"
        );
    }

    #[rstest]
    #[case("x_vault", "missing second underscore")]
    #[case("x__field", "empty app part")]
    #[case("x_app_", "empty field part")]
    #[case("y_app_field", "does not start with x_")]
    fn test_extension_key_invalid(#[case] raw_key: &str, #[case] _reason: &str) {
        let key = FieldKey::new(raw_key).expect("key must be a valid FieldKey");
        assert!(
            matches!(key.validate_as_extension(), Err(AadError::InvalidExtensionKeyFormat { .. })),
            "'{raw_key}' ({_reason}) must return InvalidExtensionKeyFormat"
        );
    }

    #[test]
    fn test_reserved_keys() {
        for &key in RESERVED_KEYS {
            let fk = FieldKey::new(key).unwrap();
            assert!(fk.is_reserved());
        }

        let fk = FieldKey::new("x_app_field").unwrap();
        assert!(!fk.is_reserved());
    }

    // -------------------------------------------------------------------------
    // ExtensionValue
    // -------------------------------------------------------------------------

    #[test]
    fn test_extension_value_string() {
        let v = ExtensionValue::string("test").unwrap();
        assert!(matches!(v, ExtensionValue::String(_)));
    }

    #[test]
    fn test_extension_value_integer() {
        let v = ExtensionValue::integer(42).unwrap();
        assert!(matches!(v, ExtensionValue::Integer(_)));
    }

    #[test]
    fn test_extension_value_nul_byte() {
        let result = ExtensionValue::string("test\0value");
        assert!(matches!(result, Err(AadError::NulByteInValue { .. })));
    }
}
