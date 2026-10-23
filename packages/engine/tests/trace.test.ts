// `og run --trace` (docs/design.md §3.4): a traced run of the lanes pile
// (tests/fixtures/trace: 2,500 boxes drawn, a system of the game's own) on
// four lanes, rendered where a renderer is, held to the Chrome trace
// format by src/trace.ts's checkTrace: every span well formed and nested in
// the one it starts in, every thread named, and spans that lie where they
// ran. The ways this could fail, each asserted below:
// - systems and passes laid end to end from their parent's start (the
//   engine recording only lengths): checkTrace refuses a thread whose
//   children are contiguous in every parent, which a synthetic trace laid
//   that way shows it does;
// - og-lanes' work invisible or folded onto one track: the physics step and
//   the CPU cull's jobs appear on two or more "lane N" threads;
// - GPU time missing where the device has timestamps (gpuTimestamps), or
//   a GPU frame whose passes leave it;
// - a take's spans moved by a different shift than the one before it, so
//   the GPU's frames, back to back across takes, overlap.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { checkTrace, type TraceDocument, type TraceEvent } from "../src/trace.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const fixture = join(pkg, "tests", "fixtures", "trace");

test("a trace laid end to end fails the check, one with recorded starts passes", () => {
  const meta: TraceEvent[] = [{ name: "thread_name", ph: "M", pid: 1, tid: 1, args: { name: "engine" } }];
  const tick = (k: number, starts: number[]): TraceEvent[] => [
    { name: `tick ${k}`, ph: "X", pid: 1, tid: 1, ts: k * 100, dur: 90 },
    ...starts.map((s, i) => ({ name: `s${i}`, ph: "X", pid: 1, tid: 1, ts: k * 100 + s, dur: 10 })),
  ];
  const doc = (starts: number[]) => ({ traceEvents: [...meta, ...Array.from({ length: 10 }, (_, k) => tick(k, starts)).flat()] });
  // What the engine wrote when it knew lengths only: from the tick's start, each after the last.
  expect(checkTrace(doc([0, 10, 20])).errors.join("\n")).toContain("laid end to end");
  // Where they ran: a gap before the first and between them.
  expect(checkTrace(doc([3, 14, 30])).errors).toEqual([]);
  // A child leaving its parent is refused too.
  expect(checkTrace(doc([3, 14, 85])).errors.join("\n")).toContain("without nesting");
});

test("og run --trace lays every span where it ran: systems, passes, GPU passes and lane jobs", () => {
  const out = join(mkdtempSync(join(tmpdir(), "og-trace-")), "trace.json");
  const r = Bun.spawnSync(
    [ogBin(), "run", "game.ts", "--headless", "--ticks", "20", "--commands", "0:pause;0:resume", "--no-check", "--trace", out, "--json"],
    { cwd: fixture, env: { ...process.env, OG_LANES: "4", OG_GFX_CULLING: "cpu" } },
  );
  const stdout = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr) + stdout);
  const run = JSON.parse(stdout.trim().split("\n").pop()!) as { tier: string; gpuTimestamps: boolean };
  const doc = JSON.parse(readFileSync(out, "utf8")) as TraceDocument;
  const check = checkTrace(doc);
  expect(check.errors).toEqual([]);

  const spans = doc.traceEvents.filter((e) => e.ph === "X") as (TraceEvent & { ts: number; dur: number; cat: string })[];
  const of = (cat: string) => spans.filter((s) => s.cat === cat);
  // Every tick, its systems inside it, the game's own among them.
  expect(of("tick").length).toBe(20);
  expect(of("system").some((s) => s.name === "Busy")).toBe(true);
  for (const s of of("system")) {
    const inside = of("tick").some((t) => t.ts <= s.ts && s.ts + s.dur <= t.ts + t.dur);
    if (!inside) throw new Error(`${s.name} at ${s.ts} lies in no tick`);
  }

  // The lanes: the physics step and the cull's jobs on the pool's threads.
  const lanes = Object.entries(check.threads).filter(([, n]) => /^lane \d+$/.test(n)).map(([tid]) => Number(tid));
  const jobs = of("lane");
  expect(new Set(jobs.map((j) => j.tid)).size).toBeGreaterThanOrEqual(2);
  expect(jobs.every((j) => lanes.includes(j.tid))).toBe(true);
  expect(jobs.some((j) => j.name === "physics")).toBe(true);

  if (run.tier === "none") return; // no renderer: no frames to time
  expect(of("render").length).toBeGreaterThan(0);
  expect(of("pass").length).toBeGreaterThan(0);
  const gpuFrames = of("gpu").filter((s) => s.name.startsWith("gpu frame"));
  if (!run.gpuTimestamps) {
    expect(gpuFrames).toEqual([]);
    return;
  }
  // Every frame rendered is timed on the GPU (the last take waits for those in flight).
  expect(gpuFrames.length).toBe(of("render").length);
  const passes = of("gpu").filter((s) => !s.name.startsWith("gpu frame"));
  const cpuNames = new Set(of("pass").map((p) => p.name));
  expect(passes.every((p) => cpuNames.has(p.name))).toBe(true);
  expect(passes.reduce((a, p) => a + p.dur, 0)).toBeGreaterThan(0);
}, 240_000);
