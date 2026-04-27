# Changelog

All notable changes to canaad-cli will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-04-27

### Changed

- Updated to `canaad-core` 1.0.0; internal call sites updated from `canonicalize`/`validate` to `canonicalize_default`/`validate_default`. CLI subcommand names and flags unchanged.

## [0.3.2] - 2026-04-13

### Changed

- Minimum supported Rust version confirmed at 1.70.

## [0.3.1] - 2026-02-20

### Changed

- Added `# Errors` sections to all `Result`-returning functions documenting error conditions.

## [0.3.0] - 2026-02-19

### Changed

- Split `main.rs` into `args.rs` (CLI types), `commands.rs` (handlers), and `io.rs` (input routing). Public CLI surface unchanged.
- Crate-level `//!` now documents exit code contract (0/1/2) and stdin-rejection invariant.

## [0.2.1] - 2026-02-19

### Added

- 23 integration tests covering all subcommands, output formats, input modes, and exit code contract.

## [0.1.0] - 2026-02-06

Initial release.

### Added

- Subcommands: `canonicalize`, `validate`, `hash`.
- Output formats: `utf8`, `hex`, `base64`, `raw`.
- Input: argument, file (`-f`), or stdin.
- Exit codes: 0 success, 1 validation error, 2 I/O error.
