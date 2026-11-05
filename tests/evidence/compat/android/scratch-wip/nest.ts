// Scratch: what runs inside the longest HandleDawnCommands slices of a trace (same thread, nested), summed by name.
import { readFileSync } from "node:fs";
const parsed = JSON.parse(readFileSync(process.argv[2]!, "utf8"));
const ev = (parsed.traceEvents ?? parsed) as { name: string; cat: string; ph: string; dur?: number; ts: number; pid: number; tid: number; args?: unknown }[];
const slices = ev.filter((e) => e.name === "WebGPUDecoderImpl::HandleDawnCommands" && (e.dur ?? 0) > 1_000_000).sort((a, b) => b.dur! - a.dur!).slice(0, 3);
for (const s of slices) {
  const inside = ev.filter((e) => e.pid === s.pid && e.tid === s.tid && e.ts >= s.ts && e.ts <= s.ts + s.dur! && e !== s);
  const by = new Map<string, { n: number; dur: number }>();
  for (const e of inside) {
    const k = `[${e.cat}] ${e.name} (${e.ph})`;
    const v = by.get(k) ?? { n: 0, dur: 0 };
    v.n++;
    v.dur += e.dur ?? 0;
    by.set(k, v);
  }
  console.log(`== slice ${(s.dur! / 1e6).toFixed(2)} s, ${inside.length} events inside`);
  for (const [k, v] of [...by].sort((a, b) => b[1].dur - a[1].dur).slice(0, 15)) console.log(`  ${(v.dur / 1e6).toFixed(2)} s x${v.n} ${k}`);
  const longest = inside.filter((e) => (e.dur ?? 0) > 200_000).sort((a, b) => b.dur! - a.dur!).slice(0, 6);
  for (const e of longest) console.log(`  long: ${(e.dur! / 1e6).toFixed(2)} s ${e.name} ${JSON.stringify(e.args ?? {}).slice(0, 200)}`);
  // Other threads of the GPU process busy during the slice.
  const others = ev.filter((e) => e.pid === s.pid && e.tid !== s.tid && e.ts >= s.ts && e.ts <= s.ts + s.dur! && (e.dur ?? 0) > 100_000);
  for (const e of others.slice(0, 6)) console.log(`  other tid ${e.tid}: ${(e.dur! / 1e6).toFixed(2)} s ${e.name}`);
}
