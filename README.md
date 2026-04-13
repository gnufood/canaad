# canaad

Deterministic AAD for AEAD.

## How it works

You build a context object — tenant, resource, purpose, optional timestamp —
and canonicalize it to bytes per [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).
Those bytes are your AAD. Pass them to your AEAD call at encryption; reconstruct
the same context and pass the same bytes at decryption. Mismatched context =
failed decryption, not silent corruption.

```rust
// the same context always produces the same bytes
let aad = AadContext::new("acme", "/doc/123", "encrypt")?
    .canonicalize()?; // → Vec<u8>, pass as AAD to your cipher
```

canaad ships as a Rust crate, CLI, and WASM package — all produce identical
output for the same input.

## Install

| Artifact | Platform | |
|----------|----------|-|
| [canaad-core](crates/canaad-core/) | Rust | `canaad-core = "0.4"` in `Cargo.toml` |
| [canaad-cli](crates/canaad-cli/) | CLI | see below |
| [canaad-wasm](crates/canaad-wasm/) | Browser / Worker | `npm install @gnufoo/canaad` |

### Rust

```toml
[dependencies]
canaad-core = "0.4"
```

### CLI

```bash
curl -fsSL https://releases.gnu.foo/canaad/install.sh | sh
# or: cargo install canaad-cli
```

### JavaScript / WASM

```bash
npm install @gnufoo/canaad
```

## Usage

### Rust

```rust
use canaad_core::{AadContext, canonicalize};

// From raw JSON
let aad = canonicalize(r#"{"v":1,"tenant":"acme","resource":"/doc/123","purpose":"encrypt"}"#)?;

// Or build it (timestamp is optional)
let aad = AadContext::new("acme", "/doc/123", "encrypt")?
    .with_timestamp(1700000000)?
    .canonicalize()?;
```

### CLI

```bash
echo '{"v":1,"tenant":"acme","resource":"/doc/123","purpose":"encrypt"}' | canaad canonicalize
canaad validate input.json
canaad hash input.json -o hex
```

### JavaScript

```typescript
import { initWasm, AadBuilder } from '@gnufoo/canaad';

await initWasm(); // call once at startup

const aad = new AadBuilder()
    .tenant("acme")
    .resource("/doc/123")
    .purpose("encrypt")
    .timestamp(1700000000) // optional
    .build(); // → Uint8Array — pass directly as AAD to your AEAD call
```

> Numbers only — no BigInt. `build()` throws on NaN, Infinity, negative, or
> fractional values.

## Development

Prerequisites: Rust 1.70+, [`just`](https://github.com/casey/just),
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (WASM only),
Rust nightly + miri component (miri only).

```bash
just ci          # full check suite (mirrors CI): lint, test, audit, msrv
just test        # native tests only
just build-wasm  # rebuild pkg/ from crates/canaad-wasm
just miri        # memory safety (nightly)
```

## Spec

[AAD specification](https://gnu.foo/specs/aad-spec/) — field constraints,
extension patterns, test vectors.

> **Security:** At decryption boundaries, surface a single opaque failure.
> Don't expose `AadError` variants to callers who don't own the input — that's
> an oracle.

## License

MIT OR Apache-2.0

[![CI](https://github.com/gnufood/canaad/actions/workflows/ci.yml/badge.svg)](https://github.com/gnufood/canaad/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canaad-core)](https://crates.io/crates/canaad-core)
[![npm](https://img.shields.io/npm/v/@gnufoo%2Fcanaad)](https://www.npmjs.com/package/@gnufood/canaad)
[![MSRV](https://img.shields.io/badge/rustc-1.70%2B-orange)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![License](https://img.shields.io/crates/l/canaad-core)](LICENSE-MIT)