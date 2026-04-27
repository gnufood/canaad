# canaad-wasm

[![npm](https://img.shields.io/npm/v/@gnufoo%2Fcanaad)](https://www.npmjs.com/package/@gnufoo/canaad)

AAD canonicalization for the browser and Cloudflare Workers. Same spec, same bytes as canaad-core.

## Initialize

WASM must be initialized once before any function calls:

```typescript
import { initWasm, canonicalize } from '@gnufoo/canaad';

await initWasm();
```

In a Worker or module context, call this at startup. Subsequent calls are no-ops.

## Canonicalize

```typescript
const bytes = canonicalize('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → Uint8Array

const str = canonicalizeString('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → string
```

## Validate

```typescript
const ok = validate('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → boolean
```

## Hash

SHA-256 of the canonical form:

```typescript
const sha = hash('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → Uint8Array (32 bytes)
```

## Build

```typescript
import { AadBuilder } from '@gnufoo/canaad';

const aad = new AadBuilder()
    .tenant("org_abc")
    .resource("secrets/db")
    .purpose("encryption")
    .timestamp(1706400000)
    .extensionString("x_vault_cluster", "us-east-1")
    .extensionInt("x_app_priority", 5)
    .build();  // → Uint8Array
```

Numbers only — no BigInt. `build()` and `buildString()` reject NaN, Infinity, negative, fractional, and values above `Number.MAX_SAFE_INTEGER`.

## Errors

Failed calls throw with a descriptive message:

```typescript
try {
    canonicalize('{"v":1}');
} catch (e) {
    console.error(e.message);  // "missing required field: tenant"
}
```

## Build from source

```bash
wasm-pack build --target web --out-dir pkg crates/canaad-wasm
```

`--target web` exports an explicit `init()` you control — works with Cloudflare Workers and Vite.
