//! Built-in AAD profiles.
//!
//! A profile layers domain-specific rules (required fields, version constraints,
//! extension allowlists) on top of the generic [`parse_object`] / [`canonicalize_object`]
//! primitives.  This module is the designated home for all first-party profiles.
//!
//! [`parse_object`]: crate::parse::parse_object
//! [`canonicalize_object`]: crate::canonicalize_object

pub(crate) mod default;
