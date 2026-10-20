// The tables of scripts/wasm-size-report.sh: per crate and per function, of
// a wasm module that keeps its name section (wasm-opt -g), with each
// group's brotli -q 11 alone beside its raw bytes.
//
//   bun scripts/wasm-size-report.ts [-n N] NAMED.wasm
//
// Brotli is what a player downloads, and raw bytes mislead about it:
// monomorphised copies of one generic function compress to little more
// than one copy. A group's "brotli alone" is its function bodies laid end
// to end and compressed by themselves; the module compresses its groups
// against each other too, so the column overstates what removing a group
// saves, most for small groups, but it ranks the groups by what they cost
// compressed.
//
// Every function is charged to the first crate its demangled name mentions
// that is not core, alloc, std or hashbrown (a std generic instantiated for
// a type, drop glue, a Vec's growth, a sort, a HashMap, is the type's
// crate; a serde visitor is the deserialiser's crate); names that mention
// only those are `std`. The binding table's exports (`og_*`, and
// wasm-bindgen's `og_* multivalue shim` and `externref shim` around them),
// into which wasm-opt inlined the export's body (the IDL's host functions,
// mostly og-runtime's code), are `og_host_web (exports)`;
// functions without a name (wasm-opt's merges of similar functions) are
// `[unnamed]`, names without a crate (memcpy, dlmalloc's entry points,
// wasm-bindgen's intrinsics) `[other]`; the data section is `[data]`.
import { readFileSync } from "node:fs";
import { brotliCompressSync, constants } from "node:zlib";

const args = process.argv.slice(2);
let top = 30;
if (args[0] === "-n") {
  top = Number(args[1]);
  args.splice(0, 2);
}
if (args.length !== 1) {
  console.error("usage: bun scripts/wasm-size-report.ts [-n N] NAMED.wasm");
  process.exit(2);
}
const wasm = readFileSync(args[0]);

let at = 0;
function leb(): number {
  let result = 0;
  let shift = 0;
  for (;;) {
    const b = wasm[at++];
    result += (b & 0x7f) * 2 ** shift;
    if ((b & 0x80) === 0) return result;
    shift += 7;
  }
}
function skipLimits() {
  const flags = leb();
  leb();
  if (flags & 1) leb();
}

let importedFunctions = 0;
const bodies: Buffer[] = [];
const names = new Map<number, string>();
let data = Buffer.alloc(0);
at = 8;
while (at < wasm.length) {
  const id = wasm[at++];
  const size = leb();
  const end = at + size;
  if (id === 2) {
    // Imports: only their count of functions matters (function indices
    // start after them).
    for (let n = leb(); n > 0; n--) {
      // (`at += leb()` would add to `at` as it was before the LEB.)
      const module = leb();
      at += module;
      const field = leb();
      at += field;
      const kind = wasm[at++];
      if (kind === 0) {
        leb();
        importedFunctions++;
      } else if (kind === 1) {
        at++;
        skipLimits();
      } else if (kind === 2) {
        skipLimits();
      } else if (kind === 3) {
        at += 2;
      } else if (kind === 4) {
        at++;
        leb();
      }
    }
  } else if (id === 10) {
    for (let n = leb(); n > 0; n--) {
      const len = leb();
      bodies.push(wasm.subarray(at, at + len));
      at += len;
    }
  } else if (id === 11) {
    data = wasm.subarray(at, end);
  } else if (id === 0) {
    const nameLen = leb();
    const section = wasm.subarray(at, at + nameLen).toString();
    at += nameLen;
    if (section === "name") {
      while (at < end) {
        const sub = wasm[at++];
        const subSize = leb();
        const subEnd = at + subSize;
        if (sub === 1) {
          for (let n = leb(); n > 0; n--) {
            const index = leb();
            const len = leb();
            names.set(index, wasm.subarray(at, at + len).toString());
            at += len;
          }
        }
        at = subEnd;
      }
    }
  }
  at = end;
}
if (names.size === 0) {
  console.error(`${args[0]} has no function names: optimise it with wasm-opt -g (scripts/wasm-size-report.sh does)`);
  process.exit(1);
}

const std = new Set(["core", "alloc", "std", "hashbrown"]);
// name[hash]: the crate hash has at least six hex digits (a name the linker
// or wasm-opt made unique ends in [1], [2], ...).
const crateRe = /([A-Za-z_][A-Za-z0-9_]*)\[[0-9a-f]{6,}\]/g;
function crateOf(name: string | undefined): string {
  if (name === undefined || /^code\[\d+\]$/.test(name)) return "[unnamed]";
  // An export of the binding table, or wasm-bindgen's shim around one.
  if (/^og_[a-z0-9_]+( multivalue shim| externref shim)?$/.test(name)) return "og_host_web (exports)";
  let sawStd = false;
  for (const m of name.matchAll(crateRe)) {
    if (!std.has(m[1])) return m[1];
    sawStd = true;
  }
  return sawStd ? "std" : "[other]";
}

function brotli(b: Buffer): number {
  return brotliCompressSync(b, {
    params: {
      [constants.BROTLI_PARAM_QUALITY]: 11,
      [constants.BROTLI_PARAM_LGWIN]: 24,
      [constants.BROTLI_PARAM_SIZE_HINT]: b.length,
    },
  }).length;
}

type Group = { items: number; bytes: number; parts: Buffer[] };
const groups = new Map<string, Group>();
const functions: { name: string; crate: string; body: Buffer }[] = [];
bodies.forEach((body, i) => {
  const name = names.get(importedFunctions + i);
  const crate = crateOf(name);
  const g = groups.get(crate) ?? { items: 0, bytes: 0, parts: [] };
  g.items++;
  g.bytes += body.length;
  g.parts.push(body);
  groups.set(crate, g);
  functions.push({ name: name ?? `code[${i}]`, crate, body });
});
groups.set("[data]", { items: 1, bytes: data.length, parts: [data] });

const total = [...groups.values()].reduce((s, g) => s + g.bytes, 0);
const rows = [...groups.entries()]
  .map(([crate, g]) => ({ crate, ...g, brotli: brotli(Buffer.concat(g.parts)) }))
  .sort((a, b) => b.brotli - a.brotli);
const brotliSum = rows.reduce((s, r) => s + r.brotli, 0);
const pct = (a: number, b: number) => `${((100 * a) / b).toFixed(2)} %`;
const num = (n: number) => n.toLocaleString("en-US");

console.log(`Per crate: code bodies and the data section, ${num(total)} B raw; "brotli alone" is each group compressed by itself (they sum to ${num(brotliSum)} B, more than the module's brotli)\n`);
console.log("| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |");
console.log("|---|---:|---:|---:|---:|---:|");
for (const r of rows) {
  console.log(`| ${r.crate} | ${num(r.items)} | ${num(r.bytes)} | ${pct(r.bytes, total)} | ${num(r.brotli)} | ${pct(r.brotli, brotliSum)} |`);
}

functions.sort((a, b) => b.body.length - a.body.length);
console.log(`\nTop ${top} functions by raw bytes\n`);
console.log("| Raw bytes | Brotli alone | Crate | Function |");
console.log("|---:|---:|---|---|");
for (const f of functions.slice(0, top)) {
  let label = f.name.replace(/\[[0-9a-f]{6,}\]/g, "").replace(/\|/g, "\\|");
  if (label.length > 160) label = `${label.slice(0, 157)}...`;
  console.log(`| ${num(f.body.length)} | ${num(brotli(f.body))} | ${f.crate} | \`${label}\` |`);
}
