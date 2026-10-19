// Instantiates a wasm-determinism module (no imports) and prints its numbers
// in the format the host binary prints: `node load.mjs <file.wasm>`.
import { readFileSync } from "node:fs";

const file = process.argv[2];
if (!file) {
  console.error("usage: node load.mjs <wasm_determinism.wasm>");
  process.exit(2);
}
const { instance } = await WebAssembly.instantiate(readFileSync(file), {});
const { math_digest, bouncing_boxes_hash, physics_hash, anim_hash } = instance.exports;
// i64 results arrive as BigInt; printed unsigned, as Rust prints u64.
const u64 = (v) => BigInt.asUintN(64, v).toString();
console.log(`math_digest ${u64(math_digest())}`);
console.log(`bouncing_boxes_hash ${u64(bouncing_boxes_hash())}`);
console.log(`physics_hash ${u64(physics_hash())}`);
console.log(`anim_hash ${u64(anim_hash())}`);
