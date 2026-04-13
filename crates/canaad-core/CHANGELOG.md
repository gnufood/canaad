# Changelog

All notable changes to canaad-core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
