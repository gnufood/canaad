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

    #[test]
    fn test_safe_int_valid() {
        assert!(SafeInt::new(0).is_ok());
        assert!(SafeInt::new(1).is_ok());
        assert!(SafeInt::new(MAX_SAFE_INTEGER).is_ok());
    }

    #[test]
    fn test_safe_int_overflow() {
        let result = SafeInt::new(MAX_SAFE_INTEGER + 1);
        assert!(matches!(result, Err(AadError::IntegerOutOfRange { .. })));
    }

    #[test]
    fn test_safe_int_negative() {
        let result = SafeInt::try_from(-1i64);
        assert!(matches!(result, Err(AadError::NegativeInteger { .. })));
    }

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

    #[test]
    fn test_field_key_invalid_chars() {
        assert!(matches!(FieldKey::new("UPPER"), Err(AadError::InvalidFieldKey { .. })));
        assert!(matches!(FieldKey::new("with-dash"), Err(AadError::InvalidFieldKey { .. })));
        assert!(matches!(FieldKey::new("with123"), Err(AadError::InvalidFieldKey { .. })));
    }

    #[test]
    fn test_extension_key_valid() {
        let key = FieldKey::new("x_vault_cluster").unwrap();
        assert!(key.validate_as_extension().is_ok());

        let key = FieldKey::new("x_myapp_region").unwrap();
        assert!(key.validate_as_extension().is_ok());

        let key = FieldKey::new("x_app_field_name").unwrap();
        assert!(key.validate_as_extension().is_ok());
    }

    #[test]
    fn test_extension_key_invalid() {
        // missing second underscore
        let key = FieldKey::new("x_vault").unwrap();
        assert!(matches!(
            key.validate_as_extension(),
            Err(AadError::InvalidExtensionKeyFormat { .. })
        ));

        // empty app part
        let key = FieldKey::new("x__field").unwrap();
        assert!(matches!(
            key.validate_as_extension(),
            Err(AadError::InvalidExtensionKeyFormat { .. })
        ));

        // empty field part
        let key = FieldKey::new("x_app_").unwrap();
        assert!(matches!(
            key.validate_as_extension(),
            Err(AadError::InvalidExtensionKeyFormat { .. })
        ));

        // doesn't start with x_
        let key = FieldKey::new("y_app_field").unwrap();
        assert!(matches!(
            key.validate_as_extension(),
            Err(AadError::InvalidExtensionKeyFormat { .. })
        ));
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
