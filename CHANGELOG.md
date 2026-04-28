# Changelog

All notable changes to the `@gnufoo/canaad` npm package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-04-28

### Changed

- Updated to `canaad-core` 2.0.0. Error strings returned from `canonicalizeDefault`, `canonicalizeObject`, and `validateDefault` for integer range and negative integer violations now include the field name (e.g. `field 'ts' negative integer not allowed: -1`).
- **Breaking:** `canonicalizeObject` and `validateObject` now enforce `[a-z_]` on all keys, returning an error for names containing uppercase letters, hyphens, dots, or digits.

## [1.0.0] - 2026-04-27

### Changed

- **Breaking:** `canonicalize` renamed to `canonicalizeDefault`; `canonicalizeString` renamed to `canonicalizeDefaultString`; `validate` renamed to `validateDefault`. The rename makes the profile layer explicit.

### Added

- `canonicalizeObject`, `canonicalizeObjectString`, `validateObject` — generic object layer; applies core rules only (size, duplicate keys, object shape, JCS) without requiring `v`, `tenant`, `resource`, or `purpose` fields.
- TypeScript test suite (vitest) covering all bindings, `AadBuilder` f64 edge cases, and the spec §10.1 known-answer vector.

---

## [0.5.1] - 2026-02-20

### Added

- `packages` field added to `meta` export: `{ npm: '@gnufoo/canaad', crates: 'canaad-core' }`. Declares registry links per tool package format spec.

---

## [0.5.0] - 2026-02-20

### Added

- `WasmNotInitializedError` — typed error class with `kind = 'WasmNotInitializedError'` discriminant, exported from `@gnufoo/canaad/tool`. Replaces bare `Error` throw in `execute()`.

---

## [0.4.3] - 2026-02-20

### Changed

- `TENANT_MAX_BYTES` and `RESOURCE_MAX_BYTES` constants extracted from magic numbers with provenance comments.
- `TextEncoder` hoisted to module level — allocated once instead of per validation call.
- Redundant `main` field removed from `package.json`.
- `canaad_wasm.d.ts` file-scope `eslint-disable` documented as known wasm-pack build artifact.

---

## [0.4.2] - 2026-02-20

### Added

- TSDoc on all exported names: `initWasm`, `isInitialized`, `toolDefinition`, `execute`, `inputSchema`, `outputSchema`, `CanaadInput`, `CanaadOutput`, `meta`.

---

## [0.4.1] - 2026-02-09

### Fixed

- `tenant` and `resource` validation corrected from character count to byte length via `TextEncoder`.
- `timestamp` and extension integers capped at `Number.MAX_SAFE_INTEGER`.

---

## [0.2.1] - 2026-02-19

### Fixed

- Zod schemas aligned with spec constraints for integer range validation.

---

## [0.2.0] - 2026-02-07

### Changed

- Added Section 9 "Integration at Decryption Boundaries" to architecture docs.

---

## [0.1.0] - 2026-02-06

Initial release.

### Added

- Functions: `canonicalize`, `canonicalizeString`, `validate`, `hash`.
- `AadBuilder` class with fluent API.
- Constants: `SPEC_VERSION`, `MAX_SAFE_INTEGER`, `MAX_SERIALIZED_BYTES`.
