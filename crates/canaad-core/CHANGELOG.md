# Changelog

All notable changes to canaad-core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-04-28

### Changed

- **Breaking:** `AadError::IntegerOutOfRange` gains a `field: String` member identifying which field triggered the error (`"v"`, `"ts"`, or the extension key name). Match arms destructuring this variant must be updated.
- **Breaking:** `AadError::NegativeInteger` gains a `field: String` member. Same requirement.
- **Breaking:** `canonicalize_object` and `validate_object` now enforce `[a-z_]` on all keys, returning `InvalidFieldKey` for names containing uppercase letters, hyphens, dots, or digits. Previously these functions accepted any key name.

### Added

- Error messages for `IntegerOutOfRange` and `NegativeInteger` now include the field name, making diagnostics self-describing without inspecting the error struct.

## [1.0.1] - 2026-04-28

### Added

- Expanded test coverage: pinned hex and SHA-256 assertions for conformance vectors §14.2–14.5 (corrected values for spec hex typos in §14.4 and §14.5); additional `parse_default` path assertions for `ts: null`, `ts: u64::MAX`, fractional `ts`, extension NUL via JSON escape, duplicate `ts` key, object/array/bool as field values, hyphen/dot/uppercase field name rejection; BOM-free output, full JCS escape set, shared-prefix and non-BMP key ordering. No behaviour changes.
- `section_10.rs` renamed to `section_14.rs`; spec places known-answer vectors in §14, not §10.

## [1.0.0] - 2026-04-27

### Changed

- **Breaking:** `parse` renamed to `parse_default`; `validate` renamed to `validate_default`; `canonicalize` renamed to `canonicalize_default`; `canonicalize_string` renamed to `canonicalize_default_string`. All four are default-profile operations — the rename makes the layer explicit.

### Added

- Generic object layer: `canonicalize_object`, `canonicalize_object_string`, `validate_object` — apply core rules (size limit, duplicate-key detection, object assertion, JCS) without requiring any specific fields. Use these to build custom profiles on top of `canaad-core`.
- `profile::default` module — explicit boundary separating the default-profile layer from the generic layer.
- `parse_object` (crate-internal) — core JSON rules extracted from `parse_aad`; both layers delegate to it.
- Fuzzing infrastructure: `cargo-fuzz` targets for `fuzz_canonicalize_object`, `fuzz_canonicalize_default`, `fuzz_parse_default` with seed corpus committed.
- `proptest` property-based tests: determinism, key-ordering, and round-trip invariants for both layers.
- `rstest` parameterized edge-case tests: field-length boundaries, NUL rejection, timestamp and integer range limits, extension key format, version constraints.
- `AadError::InvalidFloat` — new variant for WASM f64 boundary validation (NaN, Infinity, negative, fractional, above `MAX_SAFE_INTEGER`).

### Performance

- `validate_field_names`: inlined character validation before `FieldKey::new`, avoiding a heap allocation for non-extension unknown keys (e.g. keys with invalid characters). Error ordering preserved: `InvalidFieldKey` still fires before `UnknownField`.
- `extract_extensions`: removed redundant `validate_as_extension()` call — `validate_field_names` already verifies extension key format for every `x_*` key on the same parse path.

## [0.4.0] - 2026-04-13

### Fixed

- `parse_aad` narrowed from `pub` to `pub(crate)`; restores Rust 1.70 compatibility (E0446 on older toolchains).

## [0.3.0] - 2026-02-20

### Changed

- Split `tests/test_vectors.rs` (448 lines) into `test_vectors/{section_10,negative,edge_cases}.rs` by spec section.

## [0.2.4] - 2026-02-20

### Changed

- `canonicalize_context` and `canonicalize_context_string` narrowed from `pub` to `pub(crate)`; neither was re-exported through `lib.rs`.

## [0.2.3] - 2026-02-20

### Changed

- `validate()` documented as a semantic alias for `parse()` — same validation, use `validate` when you only care whether input is valid.

## [0.2.2] - 2026-02-19

### Fixed

- `with_extension` now returns `ReservedKeyAsExtension` when passed a reserved key (`v`, `tenant`, `resource`, `purpose`, `ts`). Previously returned `InvalidExtensionKeyFormat` because `is_reserved()` was never reached.

## [0.2.1] - 2026-02-19

### Fixed

- `SafeInt::try_from`: replaced bare `usize as u64` cast with `u64::try_from` for correctness on 32-bit targets.

### Changed

- Split `types.rs`, `context.rs`, and `parse.rs` into submodules. Public API unchanged.
- Duplicate-key detection replaced with a single-pass `serde_json` visitor; previous implementation allocated a `Vec<char>` and traversed the input twice.
- `ParsedAad` narrowed to `pub(crate)`; was not part of the public API.

## [0.2.0] - 2026-02-07

### Fixed

- **Breaking:** `AadContextBuilder::extension_string()` and `extension_int()` now surface validation errors through `build()` instead of silently dropping invalid values.

## [0.1.0] - 2026-02-06

Initial release.

### Added

- AAD parsing, validation, and canonicalization per RFC 8785 (JCS).
- Duplicate key detection; `serde_json` silently drops duplicates, this rejects them.
- `AadContext` with builder pattern.
- Constants: `CURRENT_VERSION`, `MAX_AAD_SIZE`, `MAX_SAFE_INTEGER`, `RESERVED_KEYS`.
- Functions: `canonicalize`, `canonicalize_string`, `parse`, `validate`.
