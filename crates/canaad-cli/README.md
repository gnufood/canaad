# canaad-cli

[![crates.io](https://img.shields.io/crates/v/canaad-cli)](https://crates.io/crates/canaad-cli)
[![MSRV](https://img.shields.io/badge/rustc-1.70%2B-orange)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

Canonicalize, validate, and hash AAD JSON from the command line. Useful for CI pipelines, test vector generation, and debugging AEAD context mismatches.

## canonicalize

```bash
canaad canonicalize '{"v":1,"tenant":"org_abc","resource":"db","purpose":"encrypt"}'
echo '...' | canaad canonicalize
canaad canonicalize -f input.json
canaad canonicalize '...' -o hex
canaad canonicalize '...' -o base64
canaad canonicalize '...' -o raw
canaad canonicalize '...' --to-file output.bin
```

## validate

```bash
canaad validate '{"v":1,"tenant":"org_abc","resource":"db","purpose":"encrypt"}'
canaad validate -f input.json
canaad validate '...' --quiet  # exit code only
```

## hash

SHA-256 of the canonical form:

```bash
canaad hash '{"v":1,"tenant":"org_abc","resource":"db","purpose":"encrypt"}'
canaad hash '...' -o base64
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation error |
| 2 | I/O error |
