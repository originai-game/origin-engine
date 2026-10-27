// The patch gate's golden replays: each trace of tests/replay/golden
// (goldens.json lists them with their project and recording) replayed with
// the og at hand (`og replay compare --json`), as the evidence that the
// engine's changes since it was recorded leave the simulation alone.
//
//   bun scripts/replay-gate.ts            compare every golden
//   bun scripts/replay-gate.ts --update   re-record every golden
//
// The og is $OG, else the one scripts/build.sh put in packages/engine/native
// for this platform. A golden fails the gate when:
// - its game changed since the recording (the report's sourceChanged): the
//   fixture is re-recorded (--update), whatever the engine did;
// - it diverged and packages/engine/changelog.json's `unreleased.simulation`
//   is not "changed" (a missing file reads as "unchanged"): the tick and the
//   keyframe's diff are printed. A change that moves the simulation on
//   purpose says so there, and the goldens are re-recorded with it.
// Exit 0 when every golden passes, 1 otherwise.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; stdout?: string; stderr?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const ROOT = join(import.meta.dir, "..");
const GOLDEN = join(ROOT, "tests/replay/golden");

interface Golden {
  trace: string;
  project: string;
  commands: string;
  ticks: number;
  seed: number;
  clock: number;
  locale?: string;
  keyframeEvery: number;
}

interface Field {
  field: string;
  local: unknown;
  remote: unknown;
}

interface Report {
  same: boolean;
  recorded: { engine: string; ticks: number };
  replayed: { engine: string; ticks: number };
  sourceChanged: boolean;
  ticksCompared: number;
  firstDivergentTick: number | null;
  keyframe: {
    tick: number;
    diff: {
      error?: string;
      components?: { entity: string; component: string; only?: string; fields?: Field[] }[];
      resources?: { resource: string; only?: string; fields?: Field[] }[];
      entities?: { onlyLocal: string[]; onlyRemote: string[] };
      events?: { event: string; local: number; remote: number }[];
      blocks?: { block: string }[];
      allocator?: boolean;
    };
  } | null;
  divergentKeyframes: number[];
}

function og(): string {
  if (process.env.OG) return process.env.OG;
  const bin = join(ROOT, "packages/engine/native", `og.${process.platform}-${process.arch}`);
  if (!existsSync(bin)) {
    process.stderr.write(`replay-gate: no og at ${bin}: run \`bun run build\` in packages/engine (or set OG)\n`);
    process.exit(1);
  }
  return bin;
}

/** The changelog's word on the unreleased simulation. */
function simulation(): string {
  const file = join(ROOT, "packages/engine/changelog.json");
  if (!existsSync(file)) return "unchanged";
  const doc = JSON.parse(readFileSync(file, "utf8")) as { unreleased?: { simulation?: string } };
  return doc.unreleased?.simulation ?? "unchanged";
}

/** What the keyframe's diff names, a line each, the first few. */
function summary(k: NonNullable<Report["keyframe"]>): string[] {
  const d = k.diff;
  if (d.error) return [`keyframe ${k.tick}: ${d.error}`];
  const out: string[] = [];
  const show = (v: unknown) => JSON.stringify(v);
  for (const c of d.components ?? []) {
    if (c.only) out.push(`${c.entity} ${c.component}: only ${c.only === "local" ? "recorded" : "replayed"}`);
    for (const f of c.fields ?? []) out.push(`${c.entity} ${c.component}.${f.field}: ${show(f.local)} → ${show(f.remote)}`);
  }
  for (const r of d.resources ?? []) {
    if (r.only) out.push(`resource ${r.resource}: only recorded`);
    for (const f of r.fields ?? []) out.push(`resource ${r.resource}.${f.field}: ${show(f.local)} → ${show(f.remote)}`);
  }
  const ents = d.entities;
  if (ents && ents.onlyLocal.length + ents.onlyRemote.length > 0) out.push(`entities only recorded: [${ents.onlyLocal.join(", ")}], only replayed: [${ents.onlyRemote.join(", ")}]`);
  for (const e of d.events ?? []) out.push(`event ${e.event}: ${e.local} queued → ${e.remote}`);
  for (const b of d.blocks ?? []) out.push(`state block ${b.block} differs`);
  if (d.allocator) out.push("the entity allocator differs");
  const shown = out.slice(0, 12);
  if (out.length > shown.length) shown.push(`... ${out.length - shown.length} more`);
  return [`keyframe ${k.tick} (recorded → replayed):`, ...shown.map((l) => `  ${l}`)];
}

const update = process.argv.includes("--update");
for (const a of process.argv.slice(2)) {
  if (a !== "--update") {
    process.stderr.write(`replay-gate: unknown argument '${a}'; usage: bun scripts/replay-gate.ts [--update]\n`);
    process.exit(2);
  }
}
const goldens = (JSON.parse(readFileSync(join(GOLDEN, "goldens.json"), "utf8")) as { goldens: Golden[] }).goldens;
const bin = og();
const sim = simulation();
let failed = 0;
for (const g of goldens) {
  const trace = join(GOLDEN, g.trace);
  const project = join(ROOT, g.project);
  const started = performance.now();
  if (update) {
    const args = ["replay", "record", project, "--commands", g.commands, "--ticks", String(g.ticks), "--seed", String(g.seed), "--clock", String(g.clock), "--keyframe-every", String(g.keyframeEvery), "--out", trace, "--json"];
    if (g.locale) args.push("--locale", g.locale);
    const r = Bun.spawnSync([bin, ...args], { cwd: ROOT, stdout: "pipe", stderr: "pipe" });
    const ms = (performance.now() - started).toFixed(0);
    if (r.exitCode !== 0) {
      failed++;
      process.stdout.write(`FAIL ${g.trace}: og replay record exited ${r.exitCode}\n${new TextDecoder().decode(r.stderr)}`);
    } else process.stdout.write(`recorded ${g.trace} (${g.project}, ${ms} ms): ${new TextDecoder().decode(r.stdout).trim()}\n`);
    continue;
  }
  const r = Bun.spawnSync([bin, "replay", "compare", trace, project, "--json"], { cwd: ROOT, stdout: "pipe", stderr: "pipe" });
  const ms = (performance.now() - started).toFixed(0);
  if (r.exitCode !== 0 && r.exitCode !== 3) {
    failed++;
    process.stdout.write(`FAIL ${g.trace}: og replay compare exited ${r.exitCode}\n${new TextDecoder().decode(r.stderr)}`);
    continue;
  }
  const report = JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").at(-1)!) as Report;
  const versions = `recorded on ${report.recorded.engine}, replayed on ${report.replayed.engine}`;
  if (report.sourceChanged) {
    failed++;
    process.stdout.write(`FAIL ${g.trace}: re-record: the fixture changed (${g.project}'s bundle is not the one recorded; bun scripts/replay-gate.ts --update)\n`);
  } else if (report.same) {
    process.stdout.write(`ok   ${g.trace}: ${report.ticksCompared} ticks the same (${versions}, ${ms} ms)\n`);
  } else if (sim === "changed") {
    process.stdout.write(`ok   ${g.trace}: diverged at tick ${report.firstDivergentTick}, and changelog.json's unreleased.simulation says changed (${versions}); re-record with --update\n`);
  } else {
    failed++;
    process.stdout.write(`FAIL ${g.trace}: the simulation diverged at tick ${report.firstDivergentTick} of ${report.recorded.ticks} (${versions}) while changelog.json's unreleased.simulation is "${sim}"\n`);
    if (report.keyframe) process.stdout.write(summary(report.keyframe).map((l) => `     ${l}\n`).join(""));
    process.stdout.write(`     a change that moves the simulation on purpose sets unreleased.simulation to "changed" and re-records (--update)\n`);
  }
}
process.exit(failed > 0 ? 1 : 0);
