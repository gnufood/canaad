//! Builder for constructing an `AadContext` with deferred validation.

use crate::context::AadContext;
use crate::error::AadError;
use crate::types::ExtensionValue;

/// Unvalidated extension value. Converted in `build()`.
#[derive(Debug)]
pub(super) enum RawExtValue {
    String(String),
    Integer(u64),
}

/// Builder for constructing an `AadContext`.
///
/// All validation is deferred to `build()`. Setters accept raw values and store
/// them without validation. Use `AadContext::builder()` to obtain an instance.
#[derive(Debug, Default)]
pub struct AadContextBuilder {
    pub(super) tenant: Option<String>,
    pub(super) resource: Option<String>,
    pub(super) purpose: Option<String>,
    pub(super) timestamp: Option<u64>,
    pub(super) extensions: Vec<(String, RawExtValue)>,
}

impl AadContextBuilder {
    /// Creates an empty builder. Setters store raw values; validation runs on `build()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores the tenant. Validated on `build()` (1-256 bytes, no NUL).
    #[must_use]
    pub fn tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = Some(tenant.into());
        self
    }

    /// Stores the resource. Validated on `build()` (1-1024 bytes, no NUL).
    #[must_use]
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Stores the purpose. Validated on `build()` (non-empty, no NUL).
    #[must_use]
    pub fn purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Stores the timestamp. Validated on `build()` (≤ `MAX_SAFE_INTEGER`).
    #[must_use]
    pub const fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Stores a string extension. Key format and value checked on `build()`.
    #[must_use]
    pub fn extension_string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extensions.push((key.into(), RawExtValue::String(value.into())));
        self
    }

    /// Stores an integer extension. Key format and value range checked on `build()`.
    #[must_use]
    pub fn extension_int(mut self, key: impl Into<String>, value: u64) -> Self {
        self.extensions.push((key.into(), RawExtValue::Integer(value)));
        self
    }

    /// Builds the `AadContext`, validating all fields.
    ///
    /// # Errors
    ///
    /// Returns an error if any required field is missing or any value is invalid.
    pub fn build(self) -> Result<AadContext, AadError> {
        let tenant = self.tenant.ok_or(AadError::MissingRequiredField { field: "tenant" })?;
        let resource = self.resource.ok_or(AadError::MissingRequiredField { field: "resource" })?;
        let purpose = self.purpose.ok_or(AadError::MissingRequiredField { field: "purpose" })?;

        let mut ctx = AadContext::new(tenant, resource, purpose)?;

        if let Some(ts) = self.timestamp {
            ctx = ctx.with_timestamp(ts)?;
        }

        let mut seen_ext_keys = std::collections::HashSet::new();
        for (key, raw) in self.extensions {
            if !seen_ext_keys.insert(key.clone()) {
                return Err(AadError::DuplicateKey { key });
            }
            let value = match raw {
                RawExtValue::String(s) => ExtensionValue::string(s)?,
                RawExtValue::Integer(i) => ExtensionValue::integer(i)?,
            };
            ctx = ctx.with_extension(key, value)?;
        }

        Ok(ctx)
    }
}
