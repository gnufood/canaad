//! WASM bindings for AAD canonicalization per RFC 8785.
//!
//! f64 inputs (timestamp, integer extensions) are validated at `build()`/`buildString()` time,
//! not at setter time. Setters store raw f64 values; boundary checks (NaN, Infinity, negative,
//! fractional, above `MAX_SAFE_INTEGER`) run when the builder is consumed.
//!
//! Serialized AAD is capped at 16 KiB. `build()` and `buildString()` return an error if the
//! canonical output exceeds this limit.
//!
//! `validate()` returns a plain bool with no error context. Use `build()` or `buildString()`
//! when the caller needs to distinguish the failure reason.

use canaad_core::{AadContext, AadError, ExtensionValue};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

// maximum safe integer (2^53 - 1); pre-computed to avoid cast precision issues
const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_991.0;

// maximum serialized AAD size (16 KiB); pre-computed since MAX_AAD_SIZE fits in u32
const MAX_SERIALIZED_BYTES_U32: u32 = 16 * 1024;

/// current AAD specification version (always 1).
#[wasm_bindgen(js_name = "SPEC_VERSION", getter)]
pub fn spec_version() -> u32 {
    1
}

/// maximum safe integer (2^53 - 1).
#[wasm_bindgen(js_name = "MAX_SAFE_INTEGER", getter)]
pub fn max_safe_integer() -> f64 {
    MAX_SAFE_INTEGER_F64
}

/// maximum serialized AAD size in bytes (16 KiB).
#[wasm_bindgen(js_name = "MAX_SERIALIZED_BYTES", getter)]
pub fn max_serialized_bytes() -> u32 {
    MAX_SERIALIZED_BYTES_U32
}

fn to_js_error(err: &AadError) -> JsError {
    JsError::new(&err.to_string())
}

/// rejects NaN, Infinity, negative (except -0.0), and fractional f64 values.
fn validate_f64_as_u64(value: f64, field: &str) -> Result<u64, AadError> {
    if value.is_nan() {
        return Err(AadError::InvalidFloat { field: field.to_owned(), reason: "NaN" });
    }
    if value.is_infinite() {
        return Err(AadError::InvalidFloat { field: field.to_owned(), reason: "Infinity" });
    }
    // -0.0 == 0.0 in IEEE 754 and JS
    if value.is_sign_negative() && value != 0.0 {
        return Err(AadError::InvalidFloat { field: field.to_owned(), reason: "negative" });
    }
    if value.fract() != 0.0 {
        return Err(AadError::InvalidFloat { field: field.to_owned(), reason: "fractional" });
    }
    if value > MAX_SAFE_INTEGER_F64 {
        return Err(AadError::InvalidFloat {
            field: field.to_owned(),
            reason: "exceeds MAX_SAFE_INTEGER",
        });
    }
    // Above 2^53-1, f64 can't represent every integer — cast is lossy; validated above.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Ok(value as u64)
}

/// parses and canonicalizes a JSON string to bytes (RFC 8785).
#[wasm_bindgen]
pub fn canonicalize(json: &str) -> Result<Vec<u8>, JsError> {
    canaad_core::canonicalize(json).map_err(|e| to_js_error(&e))
}

/// parses and canonicalizes a JSON string to a UTF-8 string.
#[wasm_bindgen(js_name = canonicalizeString)]
pub fn canonicalize_string(json: &str) -> Result<String, JsError> {
    canaad_core::canonicalize_string(json).map_err(|e| to_js_error(&e))
}

/// validates a JSON string against the AAD specification.
#[wasm_bindgen]
pub fn validate(json: &str) -> bool {
    canaad_core::validate(json).is_ok()
}

/// SHA-256 hash of the canonical JSON form.
#[wasm_bindgen]
pub fn hash(json: &str) -> Result<Vec<u8>, JsError> {
    let canonical = canaad_core::canonicalize(json).map_err(|e| to_js_error(&e))?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    Ok(hasher.finalize().to_vec())
}

/// fluent builder for AAD objects. Chain setters, call `build()` or `buildString()`.
#[wasm_bindgen]
#[must_use]
pub struct AadBuilder {
    tenant: Option<String>,
    resource: Option<String>,
    purpose: Option<String>,
    timestamp: Option<f64>,
    extensions: Vec<(String, BuilderExtValue)>,
}

/// unvalidated extension value. Converted in `build_context()`.
enum BuilderExtValue {
    String(String),
    /// stored raw; validated to u64 at build time.
    Integer(f64),
}

#[wasm_bindgen]
impl AadBuilder {
    /// creates an empty builder. Setters store raw values; validation runs on `build()`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tenant: None,
            resource: None,
            purpose: None,
            timestamp: None,
            extensions: Vec::new(),
        }
    }

    /// sets the tenant. 1-256 bytes, no NUL.
    #[wasm_bindgen]
    pub fn tenant(mut self, value: &str) -> Self {
        self.tenant = Some(value.to_string());
        self
    }

    /// sets the resource. 1-1024 bytes, no NUL.
    #[wasm_bindgen]
    pub fn resource(mut self, value: &str) -> Self {
        self.resource = Some(value.to_string());
        self
    }

    /// sets the purpose. 1+ bytes, no NUL.
    #[wasm_bindgen]
    pub fn purpose(mut self, value: &str) -> Self {
        self.purpose = Some(value.to_string());
        self
    }

    /// sets the timestamp. Validated at `build()`.
    #[wasm_bindgen]
    pub fn timestamp(mut self, ts: f64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// adds a string extension. Key format: `x_<app>_<field>`.
    #[wasm_bindgen(js_name = extensionString)]
    pub fn extension_string(mut self, key: &str, value: &str) -> Self {
        self.extensions.push((key.to_string(), BuilderExtValue::String(value.to_string())));
        self
    }

    /// adds an integer extension. Validated at `build()`.
    #[wasm_bindgen(js_name = extensionInt)]
    pub fn extension_int(mut self, key: &str, value: f64) -> Self {
        self.extensions.push((key.to_string(), BuilderExtValue::Integer(value)));
        self
    }

    fn build_context(&self) -> Result<AadContext, AadError> {
        let tenant =
            self.tenant.as_ref().ok_or(AadError::MissingRequiredField { field: "tenant" })?;
        let resource =
            self.resource.as_ref().ok_or(AadError::MissingRequiredField { field: "resource" })?;
        let purpose =
            self.purpose.as_ref().ok_or(AadError::MissingRequiredField { field: "purpose" })?;

        let mut ctx = AadContext::new(tenant, resource, purpose)?;

        if let Some(ts) = self.timestamp {
            let ts_u64 = validate_f64_as_u64(ts, "timestamp")?;
            ctx = ctx.with_timestamp(ts_u64)?;
        }

        let mut seen_ext_keys = std::collections::HashSet::new();
        for (key, value) in &self.extensions {
            if !seen_ext_keys.insert(key.clone()) {
                return Err(AadError::DuplicateKey { key: key.clone() });
            }
            let ext_value = match value {
                BuilderExtValue::String(s) => ExtensionValue::string(s)?,
                BuilderExtValue::Integer(f) => {
                    let i = validate_f64_as_u64(*f, &format!("extension '{key}'"))?;
                    ExtensionValue::integer(i)?
                }
            };
            ctx = ctx.with_extension(key, ext_value)?;
        }

        Ok(ctx)
    }

    /// builds AAD and returns canonical bytes.
    #[wasm_bindgen]
    pub fn build(&self) -> Result<Vec<u8>, JsError> {
        let ctx = self.build_context().map_err(|e| to_js_error(&e))?;
        ctx.canonicalize().map_err(|e| to_js_error(&e))
    }

    /// builds AAD and returns canonical UTF-8 string.
    #[wasm_bindgen(js_name = buildString)]
    pub fn build_string(&self) -> Result<String, JsError> {
        let ctx = self.build_context().map_err(|e| to_js_error(&e))?;
        ctx.canonicalize_string().map_err(|e| to_js_error(&e))
    }
}

impl Default for AadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
