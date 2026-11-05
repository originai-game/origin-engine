// Compares one frame's GPU cull between two browsers: the <scene>.cull.json
// that `reference-gfx.ts --cull-dump` wrote for each. Prints, per buffer,
// whether it matches and where it first differs, decoded by cull.wgsl's
// layout (scan: [0, N) the camera's visibility, then per pass g its N + 1
// prefix entries from N + g (N + 1), then the passes' block sums; args: five
// words of DrawIndexedIndirect per (pass, batch), then counters).
//
//   bun tests/evidence/compat/safari/probes/cull-compare.ts <a.cull.json> <b.cull.json>
import { readFileSync } from "node:fs";

type Cull = { instances: number; batches: number; words: Record<string, number[]>; hashes: Record<string, number[]> };
const [fa, fb] = process.argv.slice(2);
if (!fa || !fb) throw new Error("usage: cull-compare.ts <a.cull.json> <b.cull.json>");
const a = JSON.parse(readFileSync(fa, "utf8")) as Cull;
const b = JSON.parse(readFileSync(fb, "utf8")) as Cull;
const N = a.instances;
const B = a.batches;
console.log(`instances ${a.instances} / ${b.instances}, batches ${a.batches} / ${b.batches}`);

function scanAt(i: number): string {
  if (i < N) return `camera visibility of instance ${i}`;
  const q = Math.floor((i - N) / (N + 1));
  return `pass ${q}'s prefix at instance ${(i - N) % (N + 1)} (or a block sum, past the last pass)`;
}
function argsAt(i: number): string {
  const d = Math.floor(i / 5);
  const word = ["index_count", "instance_count", "first_index", "base_vertex", "first_instance"][i % 5];
  return `pass ${Math.floor(d / B)} batch ${d % B} ${word} (or a counter, past the draws)`;
}

let first = true;
for (const name of ["scan", "args"]) {
  const x = a.words[name] ?? [];
  const y = b.words[name] ?? [];
  const diffs: number[] = [];
  for (let i = 0; i < Math.max(x.length, y.length); i++) if (x[i] !== y[i]) diffs.push(i);
  if (!diffs.length) {
    console.log(`${name}: identical (${x.length} words)`);
    continue;
  }
  const at = name === "scan" ? scanAt : argsAt;
  console.log(`${name}: ${diffs.length} of ${x.length} words differ; first at ${diffs[0]}: ${at(diffs[0]!)}`);
  for (const i of diffs.slice(0, 12)) console.log(`  [${i}] ${x[i]} vs ${y[i]}  ${at(i)}`);
  first = false;
}
for (const name of Object.keys(a.hashes)) {
  const x = a.hashes[name]!;
  const y = b.hashes[name] ?? [];
  const d = x.findIndex((h, i) => h !== y[i]);
  console.log(d < 0 && x.length === y.length ? `${name}: identical (${x.length} chunks of 4 KiB)` : `${name}: first differing 4 KiB chunk ${d} (instance record ${(d * 4096) / 64})`);
}
process.exit(first ? 0 : 1);
