// Scratch: every pipeline Dawn creates in the GPU process, relative to an anchor (the HandleDawnCommands slice that starts nearest <anchorFromFirstFrameS> after og:first-frame), with labels; totals before and after it.
//   bun compiles.ts <trace.json> <tapFromFirstFrameS>
import { readFileSync } from "node:fs";
const parsed = JSON.parse(readFileSync(process.argv[2]!, "utf8"));
const ev = (parsed.traceEvents ?? parsed) as { name: string; ph: string; dur?: number; ts: number; args?: { label?: string } }[];
const ff = ev.find((e) => e.name === "og:first-frame")!.ts;
const tap = ff + Number(process.argv[3]) * 1e6;
const made = ev.filter((e) => e.ph === "X" && /^DeviceBase::APICreate(Render|Compute)Pipeline/.test(e.name)).sort((a, b) => a.ts - b.ts);
const side = (after: boolean) => made.filter((e) => (e.ts >= tap) === after);
for (const after of [false, true]) {
  const s = side(after);
  const total = s.reduce((n, e) => n + (e.dur ?? 0), 0) / 1e6;
  console.log(`${after ? "after" : "before"} the tap: ${s.length} pipelines (${s.filter((e) => /Render/.test(e.name)).length} render), ${total.toFixed(2)} s in Dawn`);
}
const by = new Map<string, { n: number; s: number }>();
for (const e of side(true)) {
  const k = e.args?.label ?? "?";
  const v = by.get(k) ?? { n: 0, s: 0 };
  v.n++;
  v.s += (e.dur ?? 0) / 1e6;
  by.set(k, v);
}
console.log("after the tap, by label:", [...by].sort((a, b) => b[1].s - a[1].s).map(([k, v]) => `${k} x${v.n} ${v.s.toFixed(2)}s`).join("; "));
const beforeTap = side(false);
const ends = beforeTap.map((e) => e.ts + (e.dur ?? 0)).sort((a, b) => a - b);
if (ends.length) console.log(`before the tap, compiles end ${((ends.at(-1)! - ff) / 1e6).toFixed(2)} s after the first frame (the last ${beforeTap.filter((e) => e.ts > ff).length} after it)`);
const names = new Map<string, number>();
for (const e of ev) if (/Pipeline/.test(e.name) && e.ph === "X") names.set(e.name, (names.get(e.name) ?? 0) + 1);
console.log("pipeline events:", [...names].map(([k, n]) => `${k} x${n}`).join("; "));
const first = side(true)[0], last = side(true).at(-1);
if (first && last) console.log(`after-tap compiles span ${((first.ts - tap) / 1e6).toFixed(2)} s .. ${((last.ts + (last.dur ?? 0) - tap) / 1e6).toFixed(2)} s from the tap`);
