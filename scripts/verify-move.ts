// Checks verbatim Rust splits (line multisets) or regroupings (token order).
//   bun scripts/verify-move.ts [--rev REV] [RENAMES] <old>[,<old>...] <new>...
//   bun scripts/verify-move.ts [--rev REV] [RENAMES] --in-place <file>...
// RENAMES: --rename OLD=NEW (repeatable), --renames FILE (one per line).
// Renames apply once, longest first, at identifier/token boundaries.
import { readFileSync } from "node:fs";

const VIS = /^pub(\((crate|super|in [\w:]+)\))?\s+/;
const SKIP = /^(mod |use |impl\b.*\{$|#\[cfg\(test\)\]$|\}$|\};$|\/\/!|#!\[)/;
type Token = [string, number];
type Pair = [string, string];
const lines = (s: string) => s.replace(/\r\n/g, "\n").split("\n");
function bag(texts: string[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const text of texts) {
    let inUse = false;
    for (const line of lines(text)) {
      let s = line.trim();
      if (inUse) {
        inUse = !s.endsWith(";");
        continue;
      }
      if (s.replace(VIS, "").startsWith("use ") && !s.endsWith(";")) {
        inUse = true;
        continue;
      }
      s = s.replace(VIS, "");
      if (s && !SKIP.test(s)) counts.set(s, (counts.get(s) ?? 0) + 1);
    }
  }
  return counts;
}
function tokens(text: string): Token[] {
  const out: Token[] = [];
  lines(text).forEach((line, i) => {
    for (const m of line.matchAll(/"(?:\\.|[^"\\])*"|\w+|\S/g)) {
      const t = m[0];
      if (")]}".includes(t) && out.at(-1)?.[0] === ",") out.pop();
      out.push([t, i + 1]);
    }
  });
  return out;
}
function renameTokens(input: Token[], pairs: Pair[]): Token[] {
  const rules = pairs
    .map(([o, n]) => [tokens(o).map(([t]) => t), tokens(n).map(([t]) => t)] as const)
    .sort(([a], [b]) => b.length - a.length);
  const out: Token[] = [];
  for (let i = 0; i < input.length;) {
    const rule = rules.find(([o]) => o.every((t, k) => input[i + k]?.[0] === t));
    if (rule) {
      out.push(...rule[1].map((t): Token => [t, input[i]![1]]));
      i += rule[0].length;
    } else out.push(input[i++]!);
  }
  return out;
}
// SequenceMatcher without junk/popularity heuristics: choose the longest
// common block, earliest in a then b, and recursively match either side.
function matching(a: Token[], b: Token[]): [number, number, number][] {
  const positions = new Map<string, number[]>();
  b.forEach(([t], j) => {
    const p = positions.get(t) ?? [];
    p.push(j);
    positions.set(t, p);
  });
  const queue = [[0, a.length, 0, b.length]],
    blocks: [number, number, number][] = [];
  while (queue.length) {
    const [alo, ahi, blo, bhi] = queue.pop() as [number, number, number, number];
    let bestI = alo,
      bestJ = blo,
      size = 0,
      prev = new Map<number, number>();
    for (let i = alo; i < ahi; i++) {
      const next = new Map<number, number>();
      for (const j of positions.get(a[i]![0]) ?? []) {
        if (j < blo) continue;
        if (j >= bhi) break;
        const k = (prev.get(j - 1) ?? 0) + 1;
        next.set(j, k);
        if (k > size) {
          bestI = i - k + 1;
          bestJ = j - k + 1;
          size = k;
        }
      }
      prev = next;
    }
    if (size) {
      blocks.push([bestI, bestJ, size]);
      if (alo < bestI && blo < bestJ) queue.push([alo, bestI, blo, bestJ]);
      if (bestI + size < ahi && bestJ + size < bhi) queue.push([bestI + size, ahi, bestJ + size, bhi]);
    }
  }
  blocks.sort(([a, b], [c, d]) => a - c || b - d);
  const out: typeof blocks = [];
  for (const [i, j, n] of blocks) {
    const p = out.at(-1);
    if (p && p[0] + p[2] === i && p[1] + p[2] === j) p[2] += n;
    else out.push([i, j, n]);
  }
  out.push([a.length, b.length, 0]);
  return out;
}
function span(toks: Token[], lo: number, hi: number): Pair {
  if (!toks.length) return ["0", "0"];
  if (hi > lo) return [String(toks[lo]![1]), String(toks[hi - 1]![1])];
  const at = String(toks[Math.min(lo, toks.length - 1)]![1]);
  return [at, at];
}
function show(rev: string, path: string): string {
  const r = Bun.spawnSync(["git", "show", `${rev}:${path}`]);
  if (r.exitCode) throw new Error(r.stderr.toString());
  return r.stdout.toString();
}
const args = process.argv.slice(2),
  raw: string[] = [];
let rev = "HEAD",
  inplace = false;
while (args[0]?.startsWith("--")) {
  const flag = args.shift();
  if (flag === "--rev") rev = args.shift()!;
  else if (flag === "--rename") raw.push(args.shift()!);
  else if (flag === "--renames")
    raw.push(
      ...lines(readFileSync(args.shift()!, "utf8"))
        .map((s) => s.trim())
        .filter((s) => s && !s.startsWith("#")),
    );
  else if (flag === "--in-place") inplace = true;
  else throw new Error(`unknown option ${flag}`);
}
const pairs: Pair[] = raw.map((s) => {
  const at = s.indexOf("=");
  if (at < 1) throw new Error(`a rename is OLD=NEW: ${JSON.stringify(s)}`);
  return [s.slice(0, at), s.slice(at + 1)];
});
if (inplace) {
  if (!args.length) throw new Error("expected --in-place <file>...");
  let differ = 0;
  for (const path of args) {
    const old = show(rev, path),
      newer = readFileSync(path, "utf8"),
      a = renameTokens(tokens(old), pairs),
      b = tokens(newer);
    const ol = lines(old),
      nl = lines(newer);
    let ai = 0,
      bi = 0;
    for (const [i, j, n] of matching(a, b)) {
      if (ai < i || bi < j) {
        differ++;
        const op = ai < i && bi < j ? "replace" : ai < i ? "delete" : "insert";
        const [oa, ob] = span(a, ai, i).map(Number) as [number, number],
          [na, nb] = span(b, bi, j).map(Number) as [number, number];
        console.log(`${path}: ${op} old ${oa}-${ob} / new ${na}-${nb}`);
        for (const l of ol.slice(oa - 1, ob)) console.log(`  - ${l.trim()}`);
        for (const l of nl.slice(na - 1, nb)) console.log(`  + ${l.trim()}`);
      }
      ai = i + n;
      bi = j + n;
    }
  }
  console.log(`${args.length} files, ${differ} differing spans`);
  process.exit(differ ? 1 : 0);
}
if (args.length < 2) throw new Error("expected <old-path>[,<old>...] <new-file>...");
const table = new Map(pairs),
  escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const pattern = table.size
  ? new RegExp(
      [...table.keys()]
        .sort((a, b) => b.length - a.length)
        .map((s) => (/^\w/.test(s) ? "(?<!\\w)" : "") + escape(s) + (/\w$/.test(s) ? "(?!\\w)" : ""))
        .join("|"),
      "g",
    )
  : undefined;
const olds = args[0]!.split(",").map((p) => {
  const s = show(rev, p);
  return pattern ? s.replace(pattern, (m) => table.get(m)!) : s;
});
const a = bag(olds),
  b = bag(args.slice(1).map((p) => readFileSync(p, "utf8")));
const difference = (a: Map<string, number>, b: Map<string, number>) =>
  [...a]
    .map(([s, n]) => [s, n - (b.get(s) ?? 0)] as const)
    .filter(([, n]) => n > 0)
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
const lost = difference(a, b),
  added = difference(b, a),
  sum = (a: Iterable<number>) => [...a].reduce((s, v) => s + v, 0);
for (const [name, d] of [
  ["only in the old file", lost],
  ["only in the new files", added],
] as const)
  for (const [line, n] of d) console.log(`${name} (${n}x): ${line}`);
console.log(
  `old ${sum(a.values())} lines, new ${sum(b.values())} lines, lost ${sum(lost.map(([, n]) => n))}, added ${sum(added.map(([, n]) => n))}`,
);
process.exit(lost.length || added.length ? 1 : 0);
