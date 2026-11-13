// `og run --profile out.md`: bun's CPU profile (`--cpu-prof`, the
// .cpuprofile JSON, its frames already mapped through the game bundle's
// source map) as tables to read: the hot functions by self time, the call
// tree by total time, each function's callers, callees and hot lines, and
// the time by file. og-cli runs it once the game process has exited.
//
//   bun src/node/profile-md.ts <in.cpuprofile> <out.md>
//
// bun's own --cpu-prof-md placed a sample that carries no position at the
// bundle's raw path with no line (`game.native.js`); here every sample counts
// at its function's mapped location, and its line where it has one.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

type CallFrame = { functionName: string; url: string; lineNumber: number; columnNumber: number };
type Node = { id: number; callFrame: CallFrame; hitCount?: number; children?: number[]; positionTicks?: { line: number; ticks: number }[] };
export type CpuProfile = { nodes: Node[]; startTime: number; endTime: number; samples: number[]; timeDeltas: number[] };

/** A function as the tables name it: its name and where it starts. */
type Fn = { key: string; name: string; location: string; file: string; self: number; total: number; samples: number; callers: Map<string, number>; callees: Map<string, number>; lines: Map<number, number> };

const NATIVE = "[native code]";

function where(f: CallFrame): { location: string; file: string } {
  if (!f.url) return { location: NATIVE, file: NATIVE };
  const file = f.url.startsWith("file:") ? fileURLToPath(f.url) : f.url;
  return { location: f.lineNumber >= 0 ? `${file}:${f.lineNumber + 1}` : file, file };
}

function nameOf(f: CallFrame): string {
  return f.functionName || "(anonymous)";
}

/** Microseconds as the tables print them. */
export function duration(us: number): string {
  if (us >= 1e6) return `${(us / 1e6).toFixed(2)}s`;
  if (us >= 1e3) return `${(us / 1e3).toFixed(1)}ms`;
  return `${Math.round(us)}us`;
}

const cell = (s: string) => `\`${s.replace(/\|/g, "\\|").replace(/`/g, "'")}\``;

export function render(p: CpuProfile): string {
  const nodes = new Map(p.nodes.map((n) => [n.id, n]));
  const parent = new Map<number, number>();
  for (const n of p.nodes) for (const c of n.children ?? []) parent.set(c, n.id);
  // Each sample's time is the delta to the next sample (the last one's, its own delta).
  const selfOf = new Map<number, number>();
  const countOf = new Map<number, number>();
  for (let i = 0; i < p.samples.length; i++) {
    const id = p.samples[i]!;
    const dt = p.timeDeltas[i + 1] ?? p.timeDeltas[i] ?? 0;
    selfOf.set(id, (selfOf.get(id) ?? 0) + Math.max(0, dt));
    countOf.set(id, (countOf.get(id) ?? 0) + 1);
  }
  const totalOf = new Map<number, number>();
  const total = (id: number): number => {
    const known = totalOf.get(id);
    if (known !== undefined) return known;
    let t = selfOf.get(id) ?? 0;
    for (const c of nodes.get(id)!.children ?? []) t += total(c);
    totalOf.set(id, t);
    return t;
  };
  const reached = new Map<number, number>();
  const samplesUnder = (id: number): number => {
    const known = reached.get(id);
    if (known !== undefined) return known;
    let n = countOf.get(id) ?? 0;
    for (const c of nodes.get(id)!.children ?? []) n += samplesUnder(c);
    reached.set(id, n);
    return n;
  };
  const root = p.nodes.find((n) => !parent.has(n.id))!;
  const all = total(root.id);
  const keyOf = (n: Node) => `${nameOf(n.callFrame)}\0${where(n.callFrame).location}`;
  const fns = new Map<string, Fn>();
  const fnOf = (n: Node): Fn => {
    const key = keyOf(n);
    let f = fns.get(key);
    if (!f) {
      const w = where(n.callFrame);
      f = { key, name: nameOf(n.callFrame), location: w.location, file: w.file, self: 0, total: 0, samples: 0, callers: new Map(), callees: new Map(), lines: new Map() };
      fns.set(key, f);
    }
    return f;
  };
  // A function's total counts each stack once, however often it recurses in it.
  const walk = (id: number, open: Set<string>) => {
    const n = nodes.get(id)!;
    const f = fnOf(n);
    const samples = countOf.get(id) ?? 0;
    f.self += selfOf.get(id) ?? 0;
    f.samples += samples;
    if (!open.has(f.key)) f.total += total(id);
    let placed = 0;
    for (const t of n.positionTicks ?? []) {
      f.lines.set(t.line, (f.lines.get(t.line) ?? 0) + t.ticks);
      placed += t.ticks;
    }
    // A sample without a position counts at the line the function starts on.
    if (samples > placed && n.callFrame.url && n.callFrame.lineNumber >= 0) {
      const line = n.callFrame.lineNumber + 1;
      f.lines.set(line, (f.lines.get(line) ?? 0) + samples - placed);
    }
    const up = parent.get(id);
    const weight = samplesUnder(id);
    if (up !== undefined && up !== root.id && weight > 0) {
      const caller = fnOf(nodes.get(up)!);
      f.callers.set(caller.name, (f.callers.get(caller.name) ?? 0) + weight);
      caller.callees.set(f.name, (caller.callees.get(f.name) ?? 0) + weight);
    }
    const inner = new Set(open).add(f.key);
    for (const c of n.children ?? []) walk(c, inner);
  };
  for (const c of root.children ?? []) walk(c, new Set());
  const pct = (us: number) => `${all > 0 ? ((us / all) * 100).toFixed(1) : "0.0"}%`;
  const list = [...fns.values()];
  const bySelf = list.filter((f) => f.self > 0).sort((a, b) => b.self - a.self || a.key.localeCompare(b.key));
  const byTotal = [...list].sort((a, b) => b.total - a.total || a.key.localeCompare(b.key));
  const out: string[] = [];
  const interval = p.samples.length > 1 ? (p.endTime - p.startTime) / p.samples.length : 0;
  out.push("# CPU Profile", "", "| Duration | Samples | Interval | Functions |", "|----------|---------|----------|----------|");
  out.push(`| ${duration(p.endTime - p.startTime)} | ${p.samples.length} | ${duration(interval)} | ${fns.size} |`, "");
  out.push(`**Top 10:** ${bySelf.slice(0, 10).map((f) => `${cell(f.name)} ${pct(f.self)}`).join(", ")}`, "");
  out.push("## Hot Functions (Self Time)", "", "| Self% | Self | Total% | Total | Function | Location |", "|------:|-----:|-------:|------:|----------|----------|");
  for (const f of bySelf.slice(0, 100)) out.push(`| ${pct(f.self)} | ${duration(f.self)} | ${pct(f.total)} | ${duration(f.total)} | ${cell(f.name)} | ${cell(f.location)} |`);
  out.push("", "## Call Tree (Total Time)", "", "Each function under its caller, indented by depth; branches under 0.5 % are left out.", "");
  out.push("| Total% | Total | Self% | Self | Function | Location |", "|-------:|------:|------:|-----:|----------|----------|");
  const tree = (id: number, depth: number) => {
    const n = nodes.get(id)!;
    if (total(id) < all * 0.005) return;
    const w = where(n.callFrame);
    out.push(`| ${pct(total(id))} | ${duration(total(id))} | ${pct(selfOf.get(id) ?? 0)} | ${duration(selfOf.get(id) ?? 0)} | ${"&nbsp;&nbsp;".repeat(depth)}${cell(nameOf(n.callFrame))} | ${cell(w.location)} |`);
    for (const c of [...(n.children ?? [])].sort((a, b) => total(b) - total(a))) tree(c, depth + 1);
  };
  for (const c of [...(root.children ?? [])].sort((a, b) => total(b) - total(a))) tree(c, 0);
  out.push("", "## Function Details", "");
  const counts = (m: Map<string, number>) => [...m].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).slice(0, 10).map(([name, n]) => `- ${cell(name)} (${n})`);
  for (const f of byTotal.filter((f) => f.total > 0).slice(0, 40)) {
    out.push(`### ${cell(f.name)}`, `${cell(f.location)} | Self: ${pct(f.self)} (${duration(f.self)}) | Total: ${pct(f.total)} (${duration(f.total)}) | Samples: ${f.samples}`, "");
    if (f.callers.size) out.push("**Called by:**", ...counts(f.callers), "");
    if (f.callees.size) out.push("**Calls:**", ...counts(f.callees), "");
    if (f.lines.size && f.file !== NATIVE) {
      const lines = [...f.lines].sort((a, b) => b[1] - a[1] || a[0] - b[0]).slice(0, 5);
      out.push("**Hot lines:**", ...lines.map(([line, n]) => `- ${cell(`${f.file}:${line}`)} (${n})`), "");
    }
  }
  const files = new Map<string, number>();
  for (const f of list) files.set(f.file, (files.get(f.file) ?? 0) + f.self);
  out.push("## Files", "", "| Self% | Self | File |", "|------:|-----:|------|");
  for (const [file, us] of [...files].filter(([, us]) => us > 0).sort((a, b) => b[1] - a[1])) out.push(`| ${pct(us)} | ${duration(us)} | ${cell(file)} |`);
  return `${out.join("\n")}\n`;
}

if (import.meta.main) {
  const [input, output] = process.argv.slice(2);
  if (!input || !output) {
    process.stderr.write("usage: bun src/node/profile-md.ts <in.cpuprofile> <out.md>\n");
    process.exit(2);
  }
  writeFileSync(output, render(JSON.parse(readFileSync(input, "utf8")) as CpuProfile));
}
