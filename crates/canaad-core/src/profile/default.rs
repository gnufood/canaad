//! Default AAD profile: enforces `v`, `tenant`, `resource`, `purpose`, optional `ts`
//! and `x_*` extensions.
//!
//! This module is the explicit boundary for everything that belongs to the
//! default profile layer.  Generic layer code (parse, canon, types) is shared
//! across profiles and lives outside this module.

// Re-exports for the default profile boundary.  These are structural markers:
// callers outside this module import from here rather than from the individual
// implementation modules directly.
pub(crate) use crate::context::AadContext;
// `AadContextBuilder` and `ParsedAad` have no crate-internal call sites that
// can import through this boundary without creating a circular dependency (both
// are consumed exclusively inside their own defining modules).  The `#[allow]`
// keeps the lint quiet until an external consumer is added.
#[allow(unused_imports)]
pub(crate) use crate::context::AadContextBuilder;
pub(crate) use crate::parse::parse_aad;
#[allow(unused_imports)]
pub(crate) use crate::parse::ParsedAad;
