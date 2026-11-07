#!/usr/bin/env node
// Run after building yarn_conformance for wasm32-wasip1 with --profile wasm.
import { WASI } from 'node:wasi';
import { readFile } from 'node:fs/promises';

const wasi = new WASI({
  version: 'preview1',
  args: process.argv.length > 2 ? process.argv.slice(2) : ['yarn_conformance'],
  env: {},
  preopens: { '.': process.cwd() },
});
const bytes = await readFile(process.argv[2] ?? 'target/wasm32-wasip1/wasm/examples/yarn_conformance.wasm');
const module = await WebAssembly.compile(bytes);
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
wasi.start(instance);
