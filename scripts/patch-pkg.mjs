#!/usr/bin/env node
// Patches wasm-pack's generated pkg/package.json with the correct npm metadata.
// wasm-pack derives the package name from the crate name and has no override mechanism.
import { readFileSync, writeFileSync } from 'node:fs';

const path = new URL('../pkg/package.json', import.meta.url).pathname;
const pkg = JSON.parse(readFileSync(path, 'utf8'));

pkg.name = '@gnufoo/canaad';
pkg.exports = { '.': { types: './canaad_wasm.d.ts', import: './canaad_wasm.js' } };
pkg.files = ['canaad_wasm.js', 'canaad_wasm.d.ts', 'canaad_wasm_bg.wasm', 'canaad_wasm_bg.wasm.d.ts'];
pkg.sideEffects = false;
pkg.homepage = 'https://gnu.foo/projects/canaad';
pkg.bugs = { email: 'bugs@gnu.foo' };
delete pkg.main;
delete pkg.collaborators;

writeFileSync(path, JSON.stringify(pkg, null, 2) + '\n');
