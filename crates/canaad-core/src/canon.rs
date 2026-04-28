//! JCS (JSON Canonicalization Scheme) serialization per RFC 8785.

use crate::error::AadError;
use crate::parse::MAX_AAD_SIZE;
use crate::profile::default::AadContext;
use serde::Serialize;
use serde_json::Value;

/// RFC 8785 canonical byte form of a `serde_json::Value`.
///
/// # Errors
///
/// Returns `SerializedTooLarge` if output exceeds 16 KiB.
/// Returns `InvalidJson` if canonicalization fails.
pub fn canonicalize_value(value: &Value) -> Result<Vec<u8>, AadError> {
    let mut buf = Vec::new();
    serde_json_canonicalizer::to_writer(value, &mut buf)
        .map_err(|e| AadError::InvalidJson { message: format!("canonicalization failed: {e}") })?;

    if buf.len() > MAX_AAD_SIZE {
        return Err(AadError::SerializedTooLarge {
            max_bytes: MAX_AAD_SIZE,
            actual_bytes: buf.len(),
        });
    }

    Ok(buf)
}

/// RFC 8785 canonical byte form of any `Serialize` value.
///
/// Serializes to `serde_json::Value` first, then passes to `canonicalize_value`.
///
/// # Errors
///
/// Returns `SerializedTooLarge` if output exceeds 16 KiB.
/// Returns `InvalidJson` if serialization or canonicalization fails.
pub fn canonicalize_serializable<T: Serialize>(value: &T) -> Result<Vec<u8>, AadError> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| AadError::InvalidJson { message: format!("serialization failed: {e}") })?;

    canonicalize_value(&json_value)
}

/// RFC 8785 canonical byte form of an `AadContext`.
///
/// # Errors
///
/// Returns `SerializedTooLarge` if output exceeds 16 KiB.
pub(crate) fn canonicalize_context(ctx: &AadContext) -> Result<Vec<u8>, AadError> {
    canonicalize_serializable(ctx)
}

/// RFC 8785 canonical UTF-8 string of an `AadContext`.
///
/// # Errors
///
/// Returns `SerializedTooLarge` if output exceeds 16 KiB.
/// Returns `InvalidJson` if the canonical bytes are not valid UTF-8.
pub(crate) fn canonicalize_context_string(ctx: &AadContext) -> Result<String, AadError> {
    let bytes = canonicalize_context(ctx)?;
    String::from_utf8(bytes).map_err(|e| AadError::InvalidJson {
        message: format!("canonicalized output is not valid UTF-8: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonicalize_sorts_keys() {
        let value = json!({
            "z": 1,
            "a": 2,
            "m": 3
        });

        let result = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(result).unwrap();

        assert_eq!(s, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_canonicalize_no_whitespace() {
        let value = json!({
            "key": "value",
            "num": 42
        });

        let result = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(result).unwrap();

        // no spaces after colons or commas (JCS requirement)
        assert!(!s.contains(": "));
        assert!(!s.contains(", "));
    }

    #[test]
    fn test_canonicalize_unicode() {
        let value = json!({
            "chinese": "组织_测试",
            "emoji": "🔐"
        });

        let result = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(result).unwrap();

        assert!(s.contains("组织_测试"));
        assert!(s.contains("🔐"));
    }

    #[test]
    fn test_canonicalize_escapes() {
        let value = json!({
            "newline": "line1\nline2",
            "quote": "say \"hello\""
        });

        let result = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(result).unwrap();

        assert!(s.contains(r#"\n"#));
        assert!(s.contains(r#"\""#));
    }

    #[test]
    fn test_canonicalize_integers() {
        let value = json!({
            "zero": 0,
            "positive": 42,
            "large": 9_007_199_254_740_991_u64
        });

        let result = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(result).unwrap();

        assert!(!s.contains(".0"));
        assert!(s.contains("9007199254740991"));
    }

    #[test]
    fn test_size_limit() {
        let large_string = "x".repeat(MAX_AAD_SIZE);
        let value = json!({ "data": large_string });

        let result = canonicalize_value(&value);
        assert!(matches!(result, Err(AadError::SerializedTooLarge { .. })));
    }

    #[test]
    fn test_canonicalize_no_bom() {
        let value = json!({"v": 1, "tenant": "org"});
        let bytes = canonicalize_value(&value).unwrap();
        assert!(
            !bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
            "canonical output must not begin with a UTF-8 BOM"
        );
    }

    #[test]
    fn test_canonicalize_all_jcs_escapes() {
        let value = json!({
            "backslash": "a\\b",
            "tab":       "a\tb",
            "cr":        "a\rb",
            "backspace": "a\u{0008}b",
            "formfeed":  "a\u{000C}b",
            "ctrl_01":   "a\u{0001}b",
            "ctrl_1f":   "a\u{001F}b",
        });
        let s = String::from_utf8(canonicalize_value(&value).unwrap()).unwrap();
        assert!(s.contains(r#""backslash":"a\\b""#), "backslash must be \\\\");
        assert!(s.contains(r#""tab":"a\tb""#), "tab must be \\t");
        assert!(s.contains(r#""cr":"a\rb""#), "CR must be \\r");
        assert!(s.contains(r#""backspace":"a\bb""#), "backspace must be \\b");
        assert!(s.contains(r#""formfeed":"a\fb""#), "form feed must be \\f");
        assert!(s.contains(r#""ctrl_01":"a\u0001b""#), "U+0001 must be \\u0001");
        assert!(s.contains(r#""ctrl_1f":"a\u001fb""#), "U+001F must be \\u001f");
    }

    #[test]
    fn test_canonicalize_shared_prefix_keys() {
        let value = json!({"abcd": 1, "ab": 2, "abc": 3, "a": 4});
        let s = String::from_utf8(canonicalize_value(&value).unwrap()).unwrap();
        assert_eq!(s, r#"{"a":4,"ab":2,"abc":3,"abcd":1}"#);
    }

    #[test]
    fn test_canonicalize_non_bmp_keys() {
        let value = json!({"🔐": 1, "a": 2, "z": 3});
        let bytes = canonicalize_value(&value).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        let pos_a = s.find(r#""a":"#).unwrap();
        let pos_z = s.find(r#""z":"#).unwrap();
        let pos_em = s.find("🔐").unwrap();
        assert!(pos_a < pos_z && pos_z < pos_em, "JCS UTF-16 order: a < z < 🔐");
    }
}
