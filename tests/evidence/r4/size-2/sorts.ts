// The standard library sorts per crate that monomorphised them (stable,
// unstable, the shared small sorts): bun sorts.ts NAMED.wasm.
import { execFileSync } from "node:child_process";
const csv = execFileSync("/tmp/twiggy/bin/twiggy", ["top", "-n", "1000000", "--format", "csv", process.argv[2]], { maxBuffer: 1 << 30 }).toString();
const by = new Map<string, [number, number]>();
const std = new Set(["core", "alloc", "std", "hashbrown"]);
for (const line of csv.split("\n").slice(1)) {
  const m = line.match(/^"?(.*?)"?,(\d+),/);
  if (!m || !/core\[[0-9a-f]+\]::slice::sort::/.test(m[1])) continue;
  let crate = "?";
  for (const c of m[1].matchAll(/([A-Za-z_][A-Za-z0-9_]*)\[[0-9a-f]{6,}\]/g)) if (!std.has(c[1])) { crate = c[1]; break; }
  const kind = m[1].match(/sort::(stable|unstable|shared)/)?.[1];
  const k = `${crate} ${kind}`;
  const s = by.get(k) ?? [0, 0]; s[0] += Number(m[2]); s[1]++; by.set(k, s);
}
for (const [k, s] of [...by].sort((a, b) => b[1][0] - a[1][0])) console.log(`${k.padEnd(30)} ${String(s[1]).padStart(5)} ${s[0].toLocaleString("en").padStart(10)}`);
