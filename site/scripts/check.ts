// What still stands in for the final page.
//
//   bun scripts/check.ts            list it
//   bun scripts/check.ts --final    exit 1 if anything blocks a deploy
//
// Blocks a deploy: a placeholder (pending media slot, pending number or
// benchmark row, a live scene not built yet) and interim media (real output
// standing in, labelled "Interim" on the page). Allowed: an estimate that
// is labelled on the page and carries a basis (data/benchmarks.json).
import bench from "../src/data/benchmarks.json";
import index from "../src/generated/media.json";
import { live } from "../src/live.ts";
import { UI_STATUS } from "../src/ui-grid.ts";

const final = process.argv.includes("--final");
const block: string[] = [];
const allowed: string[] = [];

for (const [slot, r] of Object.entries(index as Record<string, { status: string; from: { pending?: string } }>))
  if (r.status !== "final") block.push(`media ${slot}: ${r.status}${r.from.pending ? ` (${r.from.pending})` : ""}`);
if (UI_STATUS !== "final") block.push("ui grid: interim (og-ui reference frames until U8's showcases)");
for (const [name, path] of Object.entries(live)) if (!path) block.push(`live ${name}: not built yet`);

type L = { en: string };
for (const n of bench.engine as { key: string; value: string | null; pending?: boolean; label: L }[])
  if (n.pending || n.value == null) block.push(`number ${n.key}: ${n.label.en}`);
for (const r of bench.perf.rows as { engine: string; ms: number | null; pending?: boolean; estimate?: boolean; mode: L; basis?: L }[]) {
  if (r.pending || r.ms == null) block.push(`perf ${r.mode.en}: pending`);
  else if (r.estimate) (r.basis ? allowed : block).push(`perf ${r.mode.en}: estimate${r.basis ? "" : " without a basis"}`);
}
for (const s of bench.sizes as { engine: string; bytes: number | null; estimate: boolean; basis?: L }[]) {
  if (s.bytes == null) continue; // not shown on the page
  if (s.estimate) (s.basis?.en ? allowed : block).push(`size ${s.engine}: estimate${s.basis?.en ? "" : " without a basis"}`);
}

const list = (h: string, l: string[]) => (l.length ? `${h} (${l.length}):\n${l.map((x) => `  ${x}`).join("\n")}\n` : "");
process.stdout.write(list("blocks a deploy", block) + list("labelled estimates, allowed", allowed) || "nothing open\n");
if (final && block.length) process.exit(1);
