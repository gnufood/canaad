import { defineConfig } from "vitest/config";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PKG_DIR = resolve(__dirname, "../../pkg");

export default defineConfig({
  root: __dirname,
  resolve: {
    alias: {
      "../../../pkg/canaad_wasm.js": resolve(PKG_DIR, "canaad_wasm.js"),
    },
  },
  test: {
    environment: "node",
    include: ["tests/**/*.test.ts"],
    env: {
      CANAAD_WASM_PATH: resolve(PKG_DIR, "canaad_wasm_bg.wasm"),
    },
  },
});
