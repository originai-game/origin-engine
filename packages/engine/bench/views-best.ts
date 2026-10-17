// Runs bench/views.ts several times per host and view mode, each in a fresh
// process, and keeps the best (lowest) of each number: the machine is shared,
// so the minimum is the least disturbed sample. Prints one JSON document.
//
//   bun bench/views-best.ts [--runs 5]

import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const runs = Number(process.argv.includes("--runs") ? process.argv[process.argv.indexOf("--runs") + 1] : 5);
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const variants: Array<{ label: string; cmd: string[]; env?: Record<string, string> }> = [
  { label: "node host, aliased views (bun)", cmd: ["bun", "bench/views.ts", "--host", "node"] },
  { label: "node host, copied views (bun)", cmd: ["bun", "bench/views.ts", "--host", "node"], env: { OG_NODE_VIEWS: "copy" } },
  { label: "web host, wasm memory (bun)", cmd: ["bun", "bench/views.ts", "--host", "web"] },
  { label: "node host, aliased views (node)", cmd: ["node", "bench/views.ts", "--host", "node"] },
];
type Sample = { rawLoopMs: number; engineTickMs: number; lastMoveSystemMs: number; runtime: string; views: string };
const results = [];
for (const v of variants) {
  const samples: Sample[] = [];
  for (let i = 0; i < runs; i++) {
    const r = Bun.spawnSync(v.cmd, { cwd: pkg, env: { ...process.env, ...v.env } });
    if (r.exitCode !== 0) throw new Error(`${v.label}: ${new TextDecoder().decode(r.stderr)}`);
    samples.push(JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").pop()!));
  }
  const best = (k: keyof Sample) => Math.min(...samples.map((s) => s[k] as number));
  const raw = best("rawLoopMs");
  const tick = best("engineTickMs");
  const system = best("lastMoveSystemMs");
  results.push({
    variant: v.label,
    runtime: samples[0]!.runtime,
    views: samples[0]!.views,
    rawLoopMs: raw,
    engineTickMs: tick,
    moveSystemCallMs: system,
    tickOverRaw: +(tick / raw).toFixed(2),
    systemCallOverRaw: +(system / raw).toFixed(2),
  });
}
console.log(JSON.stringify({ entities: 10000, ticksPerSample: 2000, runs, results }, null, 2));
