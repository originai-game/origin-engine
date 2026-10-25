// Shallow bytes of the monomorphisation families (sorts, drop glue, serde, fmt, ...)
// in a module optimised with -g: bun generics.ts NAMED.wasm (twiggy 0.8.0 at /tmp/twiggy/bin).

import { execFileSync } from "node:child_process";
const csv = execFileSync("/home/user/.cargo/bin/twiggy", ["top", "-n", "1000000", "--format", "csv", process.argv[2]], { maxBuffer: 1 << 30 }).toString();
const groups: [string, RegExp][] = [
  ["sort::stable", /core::slice::sort::stable/],
  ["sort::unstable", /core::slice::sort::unstable/],
  ["sort::shared", /core::slice::sort::shared/],
  ["sort::select", /core::slice::sort::select/],
  ["drop_glue", /drop_glue/],
  ["serde Deserialize", /as serde(_core)?::de::|Deserialize|Visitor|deserialize/],
  ["serde Serialize", /as serde(_core)?::ser::|Serialize|serialize/],
  ["fmt Debug", /as core::fmt::Debug>::fmt/],
  ["fmt Display", /as core::fmt::Display>::fmt/],
  ["core::fmt", /core::fmt::/],
  ["hashbrown", /hashbrown::/],
  ["RawVec/grow", /RawVec|grow_one|reserve/],
  ["clone", /as core::clone::Clone>::clone/],
  ["PartialEq", /PartialEq>::eq/],
];
const sum = new Map<string, [number, number]>();
for (const line of csv.split("\n").slice(1)) {
  const m = line.match(/^"?(.*?)"?,(\d+),/);
  if (!m) continue;
  const name = m[1].replace(/\[[0-9a-f]{6,}\]/g, "");
  const b = Number(m[2]);
  for (const [g, re] of groups) if (re.test(name)) { const s = sum.get(g) ?? [0, 0]; s[0] += b; s[1]++; sum.set(g, s); break; }
}
for (const [g] of groups) { const s = sum.get(g) ?? [0, 0]; console.log(`${g.padEnd(20)} ${String(s[1]).padStart(6)} ${s[0].toLocaleString("en").padStart(12)}`); }
