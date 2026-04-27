# Run all checks (mirrors CI)
ci: lint test audit msrv

# Format check + clippy
lint:
    cargo fmt --all --check
    cargo clippy --workspace

# Auto-format
fmt:
    cargo fmt --all

# Native tests
test:
    cargo test --workspace

# Dependency vulnerability scan
audit:
    cargo audit

# Verify minimum supported Rust version
msrv:
    cargo msrv verify --manifest-path crates/canaad-core/Cargo.toml -- cargo check --locked
    cargo msrv verify --manifest-path crates/canaad-cli/Cargo.toml -- cargo check --locked

# WASM node tests (requires wasm-pack)
test-wasm:
    cargo build --package canaad-wasm --target wasm32-unknown-unknown
    wasm-pack test --node crates/canaad-wasm

# Rebuild WASM package into pkg/
build-wasm:
    wasm-pack build --target web --out-dir ../../pkg crates/canaad-wasm
    node scripts/patch-pkg.mjs

# TypeScript tests for the WASM bindings (requires pkg/ to be built first)
test-wasm-ts:
    npm --prefix crates/canaad-wasm run test

# Memory safety (requires nightly + miri component)
miri:
    cargo miri test --package canaad-core

# Run CI workflow locally via act (requires Docker)
ci-local:
    act -j ci
