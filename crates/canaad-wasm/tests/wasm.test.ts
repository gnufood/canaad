import { readFileSync } from "node:fs";
import { describe, it, expect, beforeAll } from "vitest";
import {
  initSync,
  canonicalizeDefault,
  canonicalizeDefaultString,
  validateDefault,
  canonicalizeObject,
  canonicalizeObjectString,
  validateObject,
  hash,
  AadBuilder,
  SPEC_VERSION,
  MAX_SAFE_INTEGER,
  MAX_SERIALIZED_BYTES,
} from "../../../pkg/canaad_wasm.js";

// ---------------------------------------------------------------------------
// WASM bootstrap
// ---------------------------------------------------------------------------

// CANAAD_WASM_PATH is injected by vitest.config.ts as an absolute path
const WASM_PATH: string = process.env["CANAAD_WASM_PATH"] as string;

beforeAll(() => {
  const wasmBytes = readFileSync(WASM_PATH);
  initSync({ module: wasmBytes }); // --target web requires sync init in Node.js
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

const VALID_DEFAULT_JSON =
  '{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}';
const CANONICAL_DEFAULT =
  '{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

describe("constants", () => {
  it("SPEC_VERSION() === 1", () => {
    expect(SPEC_VERSION()).toBe(1);
  });

  it("MAX_SAFE_INTEGER() === 9007199254740991", () => {
    expect(MAX_SAFE_INTEGER()).toBe(9_007_199_254_740_991);
  });

  it("MAX_SERIALIZED_BYTES() === 16384", () => {
    expect(MAX_SERIALIZED_BYTES()).toBe(16_384);
  });
});

// ---------------------------------------------------------------------------
// Default profile bindings (renamed from Pass 1)
// ---------------------------------------------------------------------------

describe("canonicalizeDefault", () => {
  it("returns correct canonical bytes for valid input", () => {
    const bytes = canonicalizeDefault(VALID_DEFAULT_JSON);
    expect(bytes).toBeInstanceOf(Uint8Array);
    const text = new TextDecoder().decode(bytes);
    expect(text).toBe(CANONICAL_DEFAULT);
  });

  it("throws for invalid JSON", () => {
    expect(() => canonicalizeDefault("not json")).toThrow();
  });

  it("throws when required fields are missing", () => {
    expect(() => canonicalizeDefault('{"v":1,"tenant":"org_abc"}')).toThrow();
  });
});

describe("canonicalizeDefaultString", () => {
  it("returns correct canonical string for valid input", () => {
    const result = canonicalizeDefaultString(VALID_DEFAULT_JSON);
    expect(result).toBe(CANONICAL_DEFAULT);
  });

  it("throws for invalid JSON", () => {
    expect(() => canonicalizeDefaultString("not json")).toThrow();
  });
});

describe("validateDefault", () => {
  it("returns true for a fully valid default AAD JSON", () => {
    expect(validateDefault(VALID_DEFAULT_JSON)).toBe(true);
  });

  it("returns false when required fields are missing", () => {
    expect(validateDefault('{"v":1,"tenant":"org_abc"}')).toBe(false);
  });

  it("returns false for invalid JSON", () => {
    expect(validateDefault("not json")).toBe(false);
  });

  it("returns false for a plain object that lacks AAD required fields", () => {
    // validateObject would accept this; validateDefault should not
    expect(validateDefault('{"z":"last","a":"first"}')).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// hash — SHA-256 known-answer test (spec §10.1)
// ---------------------------------------------------------------------------

describe("hash", () => {
  it("returns a 32-byte Uint8Array", () => {
    const bytes = hash(VALID_DEFAULT_JSON);
    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBe(32);
  });

  it("matches spec §10.1 known-answer SHA-256", () => {
    const bytes = hash(VALID_DEFAULT_JSON);
    expect(toHex(bytes)).toBe(
      "03fdc63d2f82815eb0a97e6f1a02890e152c021a795142b9c22e2b31a3bd83eb"
    );
  });

  it("throws for input that fails the default profile", () => {
    expect(() => hash('{"v":1}')).toThrow();
  });
});

// ---------------------------------------------------------------------------
// Generic object bindings (new in Pass 1)
// ---------------------------------------------------------------------------

describe("canonicalizeObject", () => {
  it("returns canonical bytes for a valid plain object", () => {
    const bytes = canonicalizeObject('{"z":"last","a":"first"}');
    expect(bytes).toBeInstanceOf(Uint8Array);
    const text = new TextDecoder().decode(bytes);
    expect(text).toBe('{"a":"first","z":"last"}');
  });

  it("applies key ordering (RFC 8785 lexicographic)", () => {
    const bytes = canonicalizeObject(
      '{"z":"last","m":"middle","a":"first"}'
    );
    const text = new TextDecoder().decode(bytes);
    expect(text).toBe('{"a":"first","m":"middle","z":"last"}');
  });

  it("throws for invalid JSON", () => {
    expect(() => canonicalizeObject("not json")).toThrow();
  });

  it("throws for a JSON array (not an object)", () => {
    expect(() => canonicalizeObject("[1,2,3]")).toThrow();
  });
});

describe("canonicalizeObjectString", () => {
  it('produces {"a":"first","z":"last"} from {"z":"last","a":"first"}', () => {
    const result = canonicalizeObjectString('{"z":"last","a":"first"}');
    expect(result).toBe('{"a":"first","z":"last"}');
  });

  it("throws for invalid JSON", () => {
    expect(() => canonicalizeObjectString("bad")).toThrow();
  });
});

describe("validateObject", () => {
  it("returns true for any valid JSON object", () => {
    expect(validateObject('{"z":"last","a":"first"}')).toBe(true);
  });

  it("returns true for an object that lacks AAD required fields", () => {
    // The point: generic object validation does not require AAD profile fields
    expect(validateObject('{"foo":"bar"}')).toBe(true);
  });

  it("returns false for a JSON array", () => {
    expect(validateObject("[1,2,3]")).toBe(false);
  });

  it("returns false for a JSON string literal", () => {
    expect(validateObject('"hello"')).toBe(false);
  });

  it("returns false for a JSON number", () => {
    expect(validateObject("42")).toBe(false);
  });

  it("returns false for invalid JSON", () => {
    expect(validateObject("not json")).toBe(false);
  });

  it("validateDefault rejects input that validateObject accepts", () => {
    const json = '{"foo":"bar"}';
    expect(validateObject(json)).toBe(true);
    expect(validateDefault(json)).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// AadBuilder — happy path
// ---------------------------------------------------------------------------

describe("AadBuilder — happy path", () => {
  it("chains tenant/resource/purpose and build() returns canonical bytes", () => {
    const bytes = new AadBuilder()
      .tenant("org_abc")
      .resource("secrets/db")
      .purpose("encryption")
      .build();
    expect(bytes).toBeInstanceOf(Uint8Array);
    const text = new TextDecoder().decode(bytes);
    expect(text).toBe(CANONICAL_DEFAULT);
  });

  it("buildString() returns correct canonical string with sorted keys", () => {
    const result = new AadBuilder()
      .tenant("org_abc")
      .resource("secrets/db")
      .purpose("encryption")
      .buildString();
    expect(result).toBe(CANONICAL_DEFAULT);
  });

  it("timestamp(1706400000) appears in output as integer", () => {
    const result = new AadBuilder()
      .tenant("org")
      .resource("res")
      .purpose("test")
      .timestamp(1_706_400_000)
      .buildString();
    expect(result).toContain('"ts":1706400000');
  });

  it("extensionString adds key-value to output", () => {
    const result = new AadBuilder()
      .tenant("org")
      .resource("res")
      .purpose("test")
      .extensionString("x_vault_cluster", "us-east-1")
      .buildString();
    expect(result).toContain('"x_vault_cluster":"us-east-1"');
  });

  it("extensionInt adds integer (not float) to output", () => {
    const result = new AadBuilder()
      .tenant("org")
      .resource("res")
      .purpose("test")
      .extensionInt("x_app_priority", 5)
      .buildString();
    // Must be 5 not 5.0
    expect(result).toContain('"x_app_priority":5');
    expect(result).not.toContain('"x_app_priority":5.0');
  });

  it("calling build() twice on the same builder returns identical results (idempotent)", () => {
    const builder = new AadBuilder()
      .tenant("org_abc")
      .resource("secrets/db")
      .purpose("encryption");

    const first = new TextDecoder().decode(builder.build());
    const second = new TextDecoder().decode(builder.build());
    expect(first).toBe(second);
  });
});

// ---------------------------------------------------------------------------
// AadBuilder — f64 edge cases that must THROW
// ---------------------------------------------------------------------------

describe("AadBuilder — f64 edge cases (must throw)", () => {
  function baseBuilder(): AadBuilder {
    return new AadBuilder().tenant("org").resource("res").purpose("test");
  }

  it("timestamp(NaN) throws", () => {
    expect(() => baseBuilder().timestamp(NaN).build()).toThrow();
  });

  it("timestamp(Infinity) throws", () => {
    expect(() => baseBuilder().timestamp(Infinity).build()).toThrow();
  });

  it("timestamp(-Infinity) throws", () => {
    expect(() => baseBuilder().timestamp(-Infinity).build()).toThrow();
  });

  it("timestamp(-1) throws", () => {
    expect(() => baseBuilder().timestamp(-1).build()).toThrow();
  });

  it("timestamp(3.7) throws (fractional)", () => {
    expect(() => baseBuilder().timestamp(3.7).build()).toThrow();
  });

  it("timestamp(9007199254740992) throws (MAX_SAFE_INTEGER + 1)", () => {
    expect(() =>
      baseBuilder()
        .timestamp(Number.MAX_SAFE_INTEGER + 1)
        .build()
    ).toThrow();
  });

  it("extensionInt('x_app_v', NaN) throws", () => {
    expect(() => baseBuilder().extensionInt("x_app_v", NaN).build()).toThrow();
  });

  it("extensionInt('x_app_v', Infinity) throws", () => {
    expect(() =>
      baseBuilder().extensionInt("x_app_v", Infinity).build()
    ).toThrow();
  });

  it("extensionInt('x_app_v', 9007199254740992) throws (MAX_SAFE_INTEGER + 1)", () => {
    expect(() =>
      baseBuilder()
        .extensionInt("x_app_v", Number.MAX_SAFE_INTEGER + 1)
        .build()
    ).toThrow();
  });
});

// ---------------------------------------------------------------------------
// AadBuilder — f64 edge cases that must SUCCEED
// ---------------------------------------------------------------------------

describe("AadBuilder — f64 edge cases (must succeed)", () => {
  function baseBuilder(): AadBuilder {
    return new AadBuilder().tenant("org").resource("res").purpose("test");
  }

  it("timestamp(0) succeeds", () => {
    expect(() => baseBuilder().timestamp(0).build()).not.toThrow();
  });

  it("timestamp(-0) succeeds (-0.0 treated as 0 in IEEE 754)", () => {
    expect(() => baseBuilder().timestamp(-0).build()).not.toThrow();
  });

  it("timestamp(9007199254740991) succeeds (exactly MAX_SAFE_INTEGER)", () => {
    expect(() =>
      baseBuilder()
        .timestamp(9_007_199_254_740_991)
        .build()
    ).not.toThrow();
  });

  it("extensionInt('x_app_v', 5) succeeds", () => {
    expect(() =>
      baseBuilder().extensionInt("x_app_v", 5).build()
    ).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// AadBuilder — error cases (missing required fields, duplicate extension key)
// ---------------------------------------------------------------------------

describe("AadBuilder — error cases", () => {
  it("missing tenant throws on build()", () => {
    expect(() =>
      new AadBuilder().resource("res").purpose("test").build()
    ).toThrow();
  });

  it("missing resource throws on build()", () => {
    expect(() =>
      new AadBuilder().tenant("org").purpose("test").build()
    ).toThrow();
  });

  it("missing purpose throws on build()", () => {
    expect(() =>
      new AadBuilder().tenant("org").resource("res").build()
    ).toThrow();
  });

  it("duplicate extension key throws on build()", () => {
    expect(() =>
      new AadBuilder()
        .tenant("org")
        .resource("res")
        .purpose("test")
        .extensionString("x_app_foo", "a")
        .extensionString("x_app_foo", "b")
        .build()
    ).toThrow();
  });
});

// ---------------------------------------------------------------------------
// AadBuilder — RFC 8785 key ordering for extensions
// ---------------------------------------------------------------------------

describe("AadBuilder — RFC 8785 key ordering for extensions", () => {
  it("extension keys are sorted lexicographically in canonical output regardless of insertion order", () => {
    // Insert x_z_first before x_a_second — RFC 8785 canonicalization sorts all
    // object keys, so x_a_second must appear before x_z_first in the output.
    const result = new AadBuilder()
      .tenant("org")
      .resource("res")
      .purpose("test")
      .extensionString("x_z_first", "val_z")
      .extensionString("x_a_second", "val_a")
      .buildString();

    const zIdx = result.indexOf("x_z_first");
    const aIdx = result.indexOf("x_a_second");
    expect(zIdx).toBeGreaterThan(-1);
    expect(aIdx).toBeGreaterThan(-1);
    // RFC 8785 sorts keys: "x_a_second" < "x_z_first" lexicographically
    expect(aIdx).toBeLessThan(zIdx);
  });
});
