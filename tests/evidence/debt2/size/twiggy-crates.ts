// Per-crate table from twiggy's `top` (shallow bytes per item) of a module
// optimised with -g (scripts/wasm-size-report.sh --keep DIR: DIR/named.wasm).
//   bun twiggy-crates.ts NAMED.wasm [WGSL_DIR]
// Items are charged to crates by the rule of scripts/wasm-size-report.ts;
// data segments are `[data]`, the name and producers sections are left out
// (the shipping module has none). With WGSL_DIR (og-render's
// $OUT_DIR/shaders), the data section is searched for every stripped WGSL
// file and the ones found are listed with their bytes.
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";

const [named, wgslDir] = process.argv.slice(2);
const csv = execFileSync(process.env.TWIGGY ?? "twiggy", ["top", "-n", "1000000", "--format", "csv", named], {
  maxBuffer: 1 << 30,
}).toString();
const std = new Set(["core", "alloc", "std", "hashbrown"]);
const crateRe = /([A-Za-z_][A-Za-z0-9_]*)\[[0-9a-f]{6,}\]/g;
function crateOf(name: string): string | null {
  if (/subsection|custom section|"producers"|"name"|"target_features"/.test(name)) return null;
  if (name.startsWith("data segment") || name.startsWith("data[")) return "[data]";
  if (/^code\[\d+\]$/.test(name)) return "[unnamed]";
  if (/^og_[a-z0-9_]+( multivalue shim| externref shim)?$/.test(name)) return "og_host_web (exports)";
  if (/^(import|export|elem|type|table|memory|global|func)\b|^(imports|exports|types|elements|globals)$/i.test(name))
    return "[wasm structure]";
  let sawStd = false;
  for (const m of name.matchAll(crateRe)) {
    if (!std.has(m[1])) return m[1];
    sawStd = true;
  }
  return sawStd ? "std" : "[other]";
}
function parseCsv(line: string): string[] {
  const out: string[] = [];
  let cur = "";
  let q = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (q) {
      if (c === '"' && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else if (c === '"') q = false;
      else cur += c;
    } else if (c === '"') q = true;
    else if (c === ",") {
      out.push(cur);
      cur = "";
    } else cur += c;
  }
  out.push(cur);
  return out;
}
const by = new Map<string, { n: number; bytes: number }>();
let total = 0;
for (const line of csv.split("\n").slice(1)) {
  if (!line) continue;
  const f = parseCsv(line);
  const name = f[0];
  const bytes = Number(f[1]);
  if (name.startsWith("Σ") || name.startsWith("...")) continue;
  const c = crateOf(name);
  if (c === null) continue;
  const g = by.get(c) ?? { n: 0, bytes: 0 };
  g.n++;
  g.bytes += bytes;
  by.set(c, g);
  total += bytes;
}
const rows = [...by.entries()].sort((a, b) => b[1].bytes - a[1].bytes);
console.log(`twiggy top, per crate (shallow bytes; names and producers left out): ${total.toLocaleString("en")} B`);
console.log("");
console.log("| Crate | Items | Shallow bytes | Share |");
console.log("|---|---:|---:|---:|");
for (const [c, g] of rows) {
  console.log(`| ${c} | ${g.n} | ${g.bytes.toLocaleString("en")} | ${((100 * g.bytes) / total).toFixed(2)} % |`);
}
if (wgslDir) {
  const wasm = readFileSync(named);
  let found = 0;
  let missing: string[] = [];
  const lines: string[] = [];
  for (const f of readdirSync(wgslDir).sort()) {
    const src = readFileSync(`${wgslDir}/${f}`);
    if (wasm.indexOf(src) >= 0) {
      found += src.length;
      lines.push(`${f} ${src.length}`);
    } else missing.push(f);
  }
  console.log("");
  console.log(`WGSL in the data section: ${lines.length} files, ${found.toLocaleString("en")} B; absent: ${missing.join(" ") || "none"}`);
}
