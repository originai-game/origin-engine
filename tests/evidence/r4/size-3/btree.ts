// bun btree.ts CSV : BTree items grouped by their (K, V) pair
import { readFileSync } from "node:fs";
const by = new Map<string, [number, number]>();
let tot = 0;
function args(s: string, at: number): string[] { // parse generic args starting at '<' index
  const out: string[] = []; let d = 0, cur = "";
  for (let i = at; i < s.length; i++) { const c = s[i];
    if (c === "<" || c === "(" || c === "[") { d++; if (d === 1) continue; }
    if (c === ">" || c === ")" || c === "]") { d--; if (d === 0) { out.push(cur.trim()); return out; } }
    if (c === "," && d === 1) { out.push(cur.trim()); cur = ""; continue; }
    cur += c; }
  return out;
}
for (const line of readFileSync(process.argv[2], "utf8").split("\n").slice(1)) {
  const m = line.match(/^"?(.*?)"?,(\d+),/); if (!m) continue;
  const n = m[1].replace(/\[[0-9a-f]{6,}\]/g, "");
  if (!/alloc::collections::btree/.test(n)) continue;
  const b = Number(m[2]); tot += b;
  let key = "?";
  const mm = n.match(/btree::(?:map::)?(?:BTreeMap|BTreeSet|IntoIter|Iter|IterMut|Range|RangeMut|entry::VacantEntry|entry::OccupiedEntry|entry::Entry|Keys|Values|ValuesMut|IntoValues|IntoKeys)</) ;
  const nr = n.match(/btree::node::(?:NodeRef|Handle<alloc::collections::btree::node::NodeRef)</);
  const idx = mm ? mm.index! + mm[0].length - 1 : nr ? n.indexOf("NodeRef<", nr.index!) + 7 : -1;
  if (idx >= 0) { const a = args(n, idx); key = nr ? `${a[1]} => ${a[2]}` : a.length >= 2 ? `${a[0]} => ${a[1]}` : `${a[0]} (set)`; }
  key = key.replace(/alloc::(vec|string|boxed|sync)::/g, "").replace(/core::option::/g, "");
  const s = by.get(key) ?? [0, 0]; s[0] += b; s[1]++; by.set(key, s);
}
console.log("total", tot, "pairs", by.size);
let u = 0; for (const [k, s] of by) if (/^u64 =>/.test(k)) u += s[0]; console.log("u64-keyed", u);
for (const [k, s] of [...by].sort((a, b) => b[1][0] - a[1][0]).slice(0, Number(process.argv[3] ?? 60))) console.log(String(s[1]).padStart(4), s[0].toLocaleString("en").padStart(8), k.slice(0, 150));
