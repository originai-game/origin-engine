// Scratch: gaps in the compositor's swaps (frames reaching the screen), relative to og:first-frame, and the long HandleDawnCommands slices beside them.
import { readFileSync } from "node:fs";
const parsed = JSON.parse(readFileSync(process.argv[2]!, "utf8"));
const ev = (parsed.traceEvents ?? parsed) as { name: string; ph: string; dur?: number; ts: number; pid: number; tid: number }[];
const ff = ev.find((e) => e.name === "og:first-frame")!.ts;
const rel = (ts: number) => ((ts - ff) / 1e6).toFixed(2);
for (const name of ["Graphics.Pipeline.DrawAndSwap", "Graphics.Pipeline"]) {
  const s = ev.filter((e) => e.name === name && e.ts > ff - 2e6).map((e) => e.ts).sort((a, b) => a - b);
  const gaps = s.slice(1).map((t, i) => ({ at: s[i]!, gap: t - s[i]! })).sort((a, b) => b.gap - a.gap).slice(0, 4);
  console.log(`${name}: ${s.length} events; longest gaps: ${gaps.map((g) => `${(g.gap / 1e6).toFixed(2)} s from ff+${rel(g.at)}`).join(", ")}`);
}
const slices = ev.filter((e) => e.name === "WebGPUDecoderImpl::HandleDawnCommands" && (e.dur ?? 0) > 1e6).sort((a, b) => a.ts - b.ts);
console.log("Dawn slices > 1 s:", slices.map((e) => `ff+${rel(e.ts)} for ${(e.dur! / 1e6).toFixed(2)} s`).join(", "));
