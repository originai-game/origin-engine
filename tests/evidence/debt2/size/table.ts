// bun table.ts LABEL=twiggy.txt ...: one markdown table, crates as rows
// (ordered by the first column), each later column with its change from the
// first. Rows under MIN bytes in every column are summed into "(the rest)".
import { readFileSync } from "node:fs";
const MIN = 20000;
const cols = process.argv.slice(2).map((a) => {
  const [label, file] = a.split("=");
  const m = new Map<string, number>();
  let total = 0;
  for (const line of readFileSync(file, "utf8").split("\n")) {
    const t = line.match(/^twiggy top.*: ([\d,]+) B/);
    if (t) total = Number(t[1].replace(/,/g, ""));
    const r = line.match(/^\| (.+?) \| (\d+) \| ([\d,]+) \|/);
    if (r) m.set(r[1], Number(r[3].replace(/,/g, "")));
  }
  return { label, m, total };
});
const names = [...cols[0].m.keys()];
for (const c of cols.slice(1)) for (const k of c.m.keys()) if (!names.includes(k)) names.push(k);
const fmt = (n: number) => n.toLocaleString("en");
const d = (n: number) => (n === 0 ? "" : ` (${n > 0 ? "+" : "−"}${fmt(Math.abs(n))})`);
console.log(`| Crate | ${cols.map((c) => c.label).join(" | ")} |`);
console.log(`|---|${cols.map(() => "---:").join("|")}|`);
const rest = cols.map(() => 0);
for (const k of names) {
  const v = cols.map((c) => c.m.get(k) ?? 0);
  if (v.every((x) => x < MIN)) {
    v.forEach((x, i) => (rest[i] += x));
    continue;
  }
  console.log(`| ${k} | ${v.map((x, i) => fmt(x) + (i > 0 ? d(x - v[0]) : "")).join(" | ")} |`);
}
console.log(`| (the rest, under ${fmt(MIN)} B) | ${rest.map((x, i) => fmt(x) + (i > 0 ? d(x - rest[0]) : "")).join(" | ")} |`);
console.log(`| **module** | ${cols.map((c, i) => `**${fmt(c.total)}**` + (i > 0 ? d(c.total - cols[0].total) : "")).join(" | ")} |`);
