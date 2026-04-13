//! `AadContext` struct, validation methods, and serialization impls.

mod builder;

pub use builder::AadContextBuilder;

use crate::canon::{canonicalize_context, canonicalize_context_string};
use crate::error::AadError;
use crate::parse::{ParsedAad, CURRENT_VERSION};
use crate::types::{ExtensionValue, Extensions, FieldKey, Purpose, Resource, SafeInt, Tenant};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

/// AAD context with all validated fields.
///
/// Represents a fully validated AAD object that can be serialized to canonical
/// form for use with AEAD algorithms. Construct via `AadContext::new`,
/// `AadContext::builder`, or `parse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AadContext {
    version: SafeInt,
    tenant: Tenant,
    resource: Resource,
    purpose: Purpose,
    timestamp: Option<SafeInt>,
    extensions: Extensions,
}

impl AadContext {
    /// Constructs an `AadContext` from the three required fields.
    ///
    /// # Errors
    ///
    /// Returns validation errors if any field violates its constraints.
    ///
    /// # Example
    ///
    /// ```
    /// use canaad_core::AadContext;
    ///
    /// let ctx = AadContext::new("org_abc", "secrets/db", "encryption")?;
    /// # Ok::<(), canaad_core::AadError>(())
    /// ```
    pub fn new(
        tenant: impl Into<String>,
        resource: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Result<Self, AadError> {
        Ok(Self {
            version: SafeInt::new(CURRENT_VERSION)?,
            tenant: Tenant::new(tenant)?,
            resource: Resource::new(resource)?,
            purpose: Purpose::new(purpose)?,
            timestamp: None,
            extensions: BTreeMap::new(),
        })
    }

    /// Creates a builder for constructing an `AadContext`.
    #[must_use]
    pub fn builder() -> AadContextBuilder {
        AadContextBuilder::new()
    }

    /// Attaches a timestamp, consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns `IntegerOutOfRange` if `ts` exceeds 2^53-1.
    pub fn with_timestamp(mut self, ts: u64) -> Result<Self, AadError> {
        self.timestamp = Some(SafeInt::new(ts)?);
        Ok(self)
    }

    /// Attaches an extension field, consuming and returning `self`.
    ///
    /// `key` must match `x_<app>_<field>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is reserved, malformed, or the value is invalid.
    pub fn with_extension(
        mut self,
        key: impl Into<String>,
        value: ExtensionValue,
    ) -> Result<Self, AadError> {
        let key = FieldKey::new(key.into())?;
        if key.is_reserved() {
            return Err(AadError::ReservedKeyAsExtension { key: key.as_str().to_owned() });
        }
        key.validate_as_extension()?;
        self.extensions.insert(key, value);
        Ok(self)
    }

    /// Attaches a string extension field, consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is malformed or the value contains NUL bytes.
    pub fn with_string_extension(
        self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, AadError> {
        self.with_extension(key, ExtensionValue::string(value)?)
    }

    /// Attaches an integer extension field, consuming and returning `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is malformed or `value` exceeds 2^53-1.
    pub fn with_int_extension(self, key: impl Into<String>, value: u64) -> Result<Self, AadError> {
        self.with_extension(key, ExtensionValue::integer(value)?)
    }

    /// Schema version (currently always 1).
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version.value()
    }

    /// Returns the tenant. Guaranteed 1-256 bytes, no NUL.
    #[must_use]
    pub fn tenant(&self) -> &str {
        self.tenant.as_str()
    }

    /// Returns the resource. Guaranteed 1-1024 bytes, no NUL.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns the purpose. Guaranteed non-empty, no NUL.
    #[must_use]
    pub fn purpose(&self) -> &str {
        self.purpose.as_str()
    }

    /// Optional unix timestamp in seconds, or `None` if not set.
    #[must_use]
    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp.map(|ts| ts.value())
    }

    /// Extension fields, sorted by key.
    #[must_use]
    pub const fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// RFC 8785 canonical byte form of this context.
    ///
    /// # Errors
    ///
    /// Returns `SerializedTooLarge` if output exceeds 16 KiB.
    pub fn canonicalize(&self) -> Result<Vec<u8>, AadError> {
        canonicalize_context(self)
    }

    /// RFC 8785 canonical UTF-8 string of this context.
    ///
    /// # Errors
    ///
    /// Returns `SerializedTooLarge` if output exceeds 16 KiB.
    pub fn canonicalize_string(&self) -> Result<String, AadError> {
        canonicalize_context_string(self)
    }
}

impl Serialize for AadContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // count fields: 4 required + optional ts + extensions
        let mut field_count = 4;
        if self.timestamp.is_some() {
            field_count += 1;
        }
        field_count += self.extensions.len();

        let mut map = serializer.serialize_map(Some(field_count))?;

        // serialize in lexicographic order for canonical output:
        // purpose, resource, tenant, ts (if present), v, x_* extensions
        map.serialize_entry("purpose", self.purpose.as_str())?;
        map.serialize_entry("resource", self.resource.as_str())?;
        map.serialize_entry("tenant", self.tenant.as_str())?;

        if let Some(ts) = &self.timestamp {
            map.serialize_entry("ts", &ts.value())?;
        }

        map.serialize_entry("v", &self.version.value())?;

        // extensions are in a BTreeMap so already sorted
        for (key, value) in &self.extensions {
            match value {
                ExtensionValue::String(s) => map.serialize_entry(key.as_str(), s)?,
                ExtensionValue::Integer(i) => map.serialize_entry(key.as_str(), &i.value())?,
            }
        }

        map.end()
    }
}

impl TryFrom<ParsedAad> for AadContext {
    type Error = AadError;

    fn try_from(parsed: ParsedAad) -> Result<Self, Self::Error> {
        Ok(Self {
            version: parsed.version,
            tenant: parsed.tenant,
            resource: parsed.resource,
            purpose: parsed.purpose,
            timestamp: parsed.timestamp,
            extensions: parsed.extensions,
        })
    }
}

#[cfg(test)]
mod tests;
