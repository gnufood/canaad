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
//! ## Architecture
//!
//! The library exposes two layers:
//!
//! - **Default-profile layer** (`parse_default`, `validate_default`,
//!   `canonicalize_default`, `canonicalize_default_string`): enforces the standard
//!   AAD field set — `v`, `tenant`, `resource`, `purpose`, optional `ts` and `x_*`
//!   extensions.
//!
//! - **Generic-object layer** (`validate_object`, `canonicalize_object`,
//!   `canonicalize_object_string`): applies core rules only (size, duplicate-key
//!   detection, object assertion, JCS canonicalization) without requiring any
//!   specific fields. Use this layer to build custom profiles on top of canaad.
//!
//! ## Quick Start
//!
//! ### Default profile
//!
//! ```rust
//! use canaad_core::{parse_default, canonicalize_default, canonicalize_default_string, AadContext};
//!
//! // Parse and validate existing JSON against the default profile
//! let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
//! let ctx = parse_default(json)?;
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
//! ### Generic object (custom profile)
//!
//! ```rust
//! use canaad_core::canonicalize_object;
//!
//! // Canonicalize any valid JSON object without profile validation
//! let json = r#"{"z":"last","a":"first"}"#;
//! let bytes = canonicalize_object(json)?;
//! # Ok::<(), canaad_core::AadError>(())
//! ```
//!
//! ## Validation Rules (default profile)
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

pub use api::{
    canonicalize_default, canonicalize_default_string, canonicalize_object,
    canonicalize_object_string, parse_default, validate_default, validate_object,
};
pub use context::{AadContext, AadContextBuilder};
pub use error::{AadError, JsonType};
pub use parse::{CURRENT_VERSION, MAX_AAD_SIZE};
pub use types::{
    ExtensionValue, Extensions, FieldKey, Purpose, Resource, SafeInt, Tenant, MAX_SAFE_INTEGER,
    RESERVED_KEYS,
};
