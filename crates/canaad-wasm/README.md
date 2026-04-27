# canaad-wasm

[![npm](https://img.shields.io/npm/v/@gnufoo%2Fcanaad)](https://www.npmjs.com/package/@gnufoo/canaad)

AAD canonicalization for the browser and Cloudflare Workers. Same spec, same bytes as canaad-core.

## Initialize

WASM must be initialized once before any function calls.

Browser / Cloudflare Workers:

```typescript
import init, { canonicalizeDefault } from '@gnufoo/canaad';

await init();
```

Node.js:

```typescript
import { readFileSync } from 'node:fs';
import { initSync, canonicalizeDefault } from '@gnufoo/canaad';

initSync({ module: readFileSync(new URL('./canaad_wasm_bg.wasm', import.meta.url)) });
```

## Default profile

```typescript
const bytes = canonicalizeDefault('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → Uint8Array

const str = canonicalizeDefaultString('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → string

const ok = validateDefault('{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}');
// → boolean
```

## Generic object layer

Canonicalize any valid JSON object — no required fields, no version check:

```typescript
const str = canonicalizeObjectString('{"z":"last","a":"first"}');
// → '{"a":"first","z":"last"}'

const ok = validateObject('{"anything":"goes"}');
// → true
```

Use this layer to build custom profiles.

## Hash

SHA-256 of the canonical form (default profile):

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
    canonicalizeDefault('{"v":1}');
} catch (e) {
    console.error(e.message);  // "missing required field: tenant"
}
```

## Build from source

```bash
wasm-pack build --target web --out-dir ../../pkg crates/canaad-wasm
```

`--target web` exports an explicit `init()` you control — works with Cloudflare Workers and Vite.
