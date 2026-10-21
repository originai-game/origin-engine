// Instantiates a wasm-determinism module and prints its numbers in the
// format the host binary prints. Each export the module has is printed (a
// one-profile build has one physics hash).
//
//   node load.mjs <file.wasm>                   a single-threaded build (no imports)
//   node load.mjs <file_bg.wasm> <glue.js>      a lanes build, bound by wasm-bindgen
//
// A lanes build (run.sh, through scripts/wasm-lanes.sh: wasm atomics, the
// shared memory imported, og-lanes' rayon with wasm-bindgen-rayon) is
// instantiated through its wasm-bindgen glue (--target web), which creates
// the shared memory and runs the thread setup. Its wasm-bindgen-rayon
// snippet listens for a pool Worker's init message on `self`, which node
// has not: an EventTarget stands in. No pool is started, so og-lanes runs
// every call on the calling thread's one-thread rayon pool, through
// rapier's parallel build.
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const [file, glueFile] = process.argv.slice(2);
if (!file) {
  console.error("usage: node load.mjs <wasm_determinism.wasm> [<wasm-bindgen glue.js>]");
  process.exit(2);
}
let exports;
if (glueFile) {
  globalThis.self ??= new EventTarget();
  const glue = await import(pathToFileURL(glueFile).href);
  exports = glue.initSync({ module: readFileSync(file) });
} else {
  ({ exports } = await WebAssembly.instantiate(readFileSync(file), {}).then((r) => r.instance));
}
// i64 results arrive as BigInt; printed unsigned, as Rust prints u64.
const u64 = (v) => BigInt.asUintN(64, v).toString();
for (const name of ["math_digest", "bouncing_boxes_hash", "physics3d_hash", "physics2d_hash", "anim_hash"]) {
  const f = exports[name];
  if (typeof f === "function") console.log(`${name} ${u64(f())}`);
}
process.exit(0);
