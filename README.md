# canaad

[![CI](https://github.com/gnufood/canaad/actions/workflows/ci.yml/badge.svg)](https://github.com/gnufood/canaad/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/canaad-core)](https://crates.io/crates/canaad-core)
[![npm](https://img.shields.io/npm/v/@gnufoo%2Fcanaad)](https://www.npmjs.com/package/@gnufood/canaad)
[![MSRV](https://img.shields.io/badge/rustc-1.70%2B-orange)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![License](https://img.shields.io/crates/l/canaad-core)](LICENSE-MIT)  
Deterministic AAD for AEAD.

[Reference Spec](https://gnu.foo/specs/aad-spec/) 

## How it works

canaad has two layers:

- **Default-profile layer** — validates the standard field set (`v`, `tenant`, `resource`,
  `purpose`, optional `ts` and `x_*` extensions), then produces an RFC 8785 canonical
  byte string. Use `canonicalize_default` / `canonicalizeDefault` in your application code.

- **Generic-object layer** — applies core rules only (size limit, duplicate-key rejection,
  object assertion, JCS canonicalization) without requiring any specific fields. Use
  `canonicalize_object` / `canonicalizeObject` when building custom profiles on top of canaad.

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
| [canaad-core](crates/canaad-core/) | Rust | `canaad-core = "1.0"` in `Cargo.toml` |
| [canaad-cli](crates/canaad-cli/) | CLI | see below |
| [canaad-wasm](crates/canaad-wasm/) | Browser / Worker | `npm install @gnufoo/canaad` |

### Rust

```toml
[dependencies]
canaad-core = "1.0"
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
use canaad_core::{AadContext, canonicalize_default, canonicalize_object};

// Default profile: validates v, tenant, resource, purpose
let aad = canonicalize_default(r#"{"v":1,"tenant":"acme","resource":"/doc/123","purpose":"encrypt"}"#)?;

// Or build from scratch (timestamp is optional)
let aad = AadContext::new("acme", "/doc/123", "encrypt")?
    .with_timestamp(1700000000)?
    .canonicalize()?;

// Generic object: core rules only, no required fields
let aad = canonicalize_object(r#"{"z":"last","a":"first"}"#)?;
```

### CLI

```bash
echo '{"v":1,"tenant":"acme","resource":"/doc/123","purpose":"encrypt"}' | canaad canonicalize
canaad validate input.json
canaad hash input.json -o hex
```

### JavaScript

```typescript
import { initWasm, AadBuilder, canonicalizeDefault, canonicalizeObject } from '@gnufoo/canaad';

await initWasm(); // call once at startup

// Default profile
const aad = new AadBuilder()
    .tenant("acme")
    .resource("/doc/123")
    .purpose("encrypt")
    .timestamp(1700000000) // optional
    .build(); // → Uint8Array — pass directly as AAD to your AEAD call

// Or canonicalize existing JSON with the default profile
const aad2 = canonicalizeDefault('{"v":1,"tenant":"acme","resource":"/doc/123","purpose":"encrypt"}');

// Generic object: core rules only, no required fields
const aad3 = canonicalizeObject('{"z":"last","a":"first"}');
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
## License

MIT OR Apache-2.0
