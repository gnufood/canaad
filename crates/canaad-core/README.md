# canaad-core

[![crates.io](https://img.shields.io/crates/v/canaad-core)](https://crates.io/crates/canaad-core)
[![MSRV](https://img.shields.io/badge/rustc-1.70%2B-orange)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

Parse, validate, and canonicalize AAD contexts per [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785). Produces deterministic bytes for use as AEAD additional authenticated data — same input, same bytes, across Rust, WASM, and any conforming implementation.

## Use it

### Default profile

Parse existing JSON:

```rust
use canaad_core::parse_default;

let json = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;
let ctx = parse_default(json)?;
let canonical = ctx.canonicalize_string()?;
```

Build from scratch:

```rust
use canaad_core::AadContext;

let ctx = AadContext::new("org_abc", "secrets/db", "encryption")?
    .with_timestamp(1706400000)?
    .with_string_extension("x_vault_cluster", "us-east-1")?;

let bytes = ctx.canonicalize()?;
```

Use the builder if you prefer:

```rust
use canaad_core::AadContext;

let ctx = AadContext::builder()
    .tenant("org_abc")
    .resource("secrets/db")
    .purpose("encryption")
    .extension_string("x_vault_cluster", "us-east-1")
    .build()?;
```

Builder defers all validation to `build()` — invalid extensions surface as errors, not silent drops.

### Generic object layer

Canonicalize any valid JSON object without profile validation — no required fields, no version check:

```rust
use canaad_core::canonicalize_object;

let bytes = canonicalize_object(r#"{"z":"last","a":"first"}"#)?;
// → {"a":"first","z":"last"} (keys sorted per JCS)
```

Use this layer to build custom profiles on top of `canaad-core`.

## What it checks

- `v` must be 1
- `tenant`: 1–256 bytes, no NUL
- `resource`: 1–1024 bytes, no NUL
- `purpose`: 1+ bytes, no NUL
- `ts`: optional, 0 to 2^53−1
- extensions: `x_<app>_<field>` pattern, values are strings or integers
- no duplicate keys (custom single-pass scanner, not serde_json)
- 16 KiB max serialized size

All 17 error variants are strongly typed via `AadError`. See [docs.rs/canaad-core](https://docs.rs/canaad-core) for the full API.

## Spec

[gnu.foo/specs/aad-spec](https://gnu.foo/specs/aad-spec/) — field constraints, extension patterns, test vectors.
