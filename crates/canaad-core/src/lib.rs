//! Core library for AAD canonicalization per RFC 8785 (JCS).
//!
//! Provides deterministic serialization and contextual binding for AEAD
//! additional authenticated data. Mitigates confused deputy attacks,
//! cross-tenant decryption, and purpose confusion by binding ciphertext
//! to tenant, resource, and purpose fields with a canonical byte representation.
//!
//! See the [AAD specification](https://gnu.foo/specs/aad-spec/) for the full schema,
//! constraint set, and test vectors. Architecture and integration guidance at
//! [gnu.foo/projects/canaad](https://gnu.foo/projects/canaad).
//!
//! ## Quick Start
//!
//! ```rust
//! use canaad_core::{parse, canonicalize, canonicalize_string, AadContext};
//!
//! // Parse and validate existing JSON
//! let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
//! let ctx = parse(json)?;
//! let canonical = ctx.canonicalize_string()?;
//!
//! // Or build from scratch
//! let ctx = AadContext::new("org_abc", "secrets/db", "encryption")?
//!     .with_timestamp(1706400000)?
//!     .with_string_extension("x_vault_cluster", "us-east-1")?;
//!
//! let bytes = ctx.canonicalize()?;
//! # Ok::<(), canaad_core::AadError>(())
//! ```
//!
//! ## Validation Rules
//!
//! - Version (`v`): Must be 1
//! - Tenant: 1-256 bytes, no NUL bytes
//! - Resource: 1-1024 bytes, no NUL bytes
//! - Purpose: 1+ bytes, no NUL bytes
//! - Timestamp (`ts`): Optional, 0 to 2^53-1
//! - Extension keys: Must match pattern `x_<app>_<field>` where app is `[a-z]+` and field is `[a-z_]+`
//! - All integers: 0 to 2^53-1 (JavaScript safe integer range)
//! - Total serialized size: Maximum 16 KiB
//! - No duplicate keys allowed

mod api;
mod canon;
mod context;
mod error;
mod parse;
mod types;

#[cfg(test)]
mod tests;

pub use api::{canonicalize, canonicalize_string, parse, validate};
pub use context::{AadContext, AadContextBuilder};
pub use error::{AadError, JsonType};
pub use parse::{CURRENT_VERSION, MAX_AAD_SIZE};
pub use types::{
    ExtensionValue, Extensions, FieldKey, Purpose, Resource, SafeInt, Tenant, MAX_SAFE_INTEGER,
    RESERVED_KEYS,
};
