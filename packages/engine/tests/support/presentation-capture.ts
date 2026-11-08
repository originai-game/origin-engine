// One capture of a project at a tick, produced one of four ways (the
// presentation gate, docs/design.md §2.4; packages/engine/tests/presentation.test.ts):
//
//   every     a display frame on every tick from tick 0, then the capture
//   capture   no display frame: the ticks are stepped, then the capture
//   restore   a fork: display frames throughout; snapshots at --fork-at (R)
//             and --restore-at (A); on to --ticks and a capture (`first`);
//             R restored and played on past A without the inputs (another
//             branch); A restored and played on to --ticks with them, and
//             the capture (the answer)
//   synctest  GGPO's SyncTest for presentation: on every tick from
//             --restore-at, the snapshot of --rollback ticks back restored
//             and those ticks played again; the world and presentation
//             hashes they reach must be the first pass's
//   display   the display's own frames, not captures: a frame on every tick
//             from --fork-at to --ticks, then another branch of 24 ticks
//             from the fork's snapshot without the inputs, the fork restored
//             and the same ticks drawn again; every frame must be the first
//             pass's byte for byte (`checked`, `mismatches`); with
//             --frames <dir>, the first pass's frames as <tick>.rgba there
//             (tests/web/presentation-fork.ts compares the web's with them)
//   ticks     a display frame on every tick, and each tick's world and
//             presentation hash (`ticks`, from tick 1), then `dump_motion`
//             (`motion`): what the web's run of the same project must reach
//             (tests/web/motion-ticks.ts)
//
//   bun tests/support/presentation-capture.ts --project <dir> --mode every|capture|restore|synctest|display|ticks
//       --ticks N [--restore-at A] [--fork-at R] [--rollback K] [--size WxH]
//       [--inputs "tick:action[:args];..."] [--out <file.png>] [--frames <dir>]
//
// Inputs are probe steps sent before the tick they name, in every mode and
// on every pass over a tick, so a restored run presses the same keys on the
// same ticks as the first. The last stdout line is JSON: the tick, the
// world hash, the PNG's SHA-256 (and `first`'s with --mode restore), the
// presentation hash, and with --mode synctest the ticks checked and the
// mismatches.
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback?: string): string => {
  const i = argv.indexOf(name);
  if (i >= 0 && argv[i + 1] !== undefined) return argv[i + 1]!;
  if (fallback === undefined) throw new Error(`${name} is required`);
  return fallback;
};
const project = opt("--project");
const mode = opt("--mode");
const ticks = Number(opt("--ticks"));
const [width, height] = opt("--size", "960x540").split("x").map(Number) as [number, number];
const out = argv.includes("--out") ? opt("--out") : undefined;
const inputs = new Map<number, string[]>();
for (const step of opt("--inputs", "").split(";").filter(Boolean)) {
  const at = step.indexOf(":");
  const tick = Number(step.slice(0, at));
  inputs.set(tick, [...(inputs.get(tick) ?? []), `0:${step.slice(at + 1)}`]);
}

const { loadNativeHost, native, attachRenderer } = await import("../../src/node/index.ts");
loadNativeHost();
// The game's own module beside this script, so its "@origingame/engine" is
// this checkout's package, as og run's is.
const here = dirname(fileURLToPath(import.meta.url));
const copy = join(here, `.presentation-main-${process.pid}.ts`);
copyFileSync(join(project, "src", "main.ts"), copy);
const game = await import(copy)
  .then((m) => (m as { default: GameToken }).default)
  .finally(() => rmSync(copy, { force: true }));
const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const text = existsSync(join(project, "project.ogt")) ? readFileSync(join(project, "project.ogt"), "utf8") : undefined;
const profile = text === undefined ? undefined : /^\s*profile\s+"(2d|3d)"/m.exec(text)?.[1];
const engine = must(createEngine(game, { root: project, ...(text ? { project: text } : {}), ...(profile ? { profile: profile as "2d" | "3d" } : {}), deferStart: true }));
native().og_node_scripted_frames(engine.id, true);
const attached = attachRenderer(engine.id, width, height);
if (!attached.ok) throw new Error(attached.reason);
if (attached.value === "none") throw new Error(`no renderer: ${attached.adapterError}`);
must(engine.advance(0));

const probe = engine.probe as unknown as {
  commands(script: string): unknown;
  advance(steps: number): unknown;
  capture(width: number, height: number): { tick: number; png: string } | null;
  snapshot(): unknown;
  restore(id: string): unknown;
};
const tickMs = 1000 / engine.rate;
const tick = () => engine.world.tick;

/** Step to `until`, a display frame per tick or none. */
function stepTo(until: number, frames: boolean): void {
  while (tick() < until) {
    const due = inputs.get(tick());
    if (due) probe.commands(due.join(";"));
    if (frames) {
      const f = engine.frame(tickMs);
      if (!f.ok) throw new Error(f.reason);
    } else probe.advance(1);
  }
  for (const l of engine.takeLog()) if (l.warning) process.stderr.write(`WARN tick ${l.tick}: ${l.text}\n`);
}

function shot(): { png: Uint8Array; sha: string } {
  const c = probe.capture(width, height);
  if (!c) throw new Error("the renderer produced no frame");
  const png = Uint8Array.from(atob(c.png), (ch) => ch.charCodeAt(0));
  return { png, sha: createHash("sha256").update(png).digest("hex") };
}

const restoreAt = Number(opt("--restore-at", String(Math.max(1, ticks - 16))));
const forkAt = Number(opt("--fork-at", String(Math.floor(restoreAt / 2))));
const rollback = Number(opt("--rollback", "4"));
type Snap = { id: string; tick: number; hash: string; presentation: string };

let first: string | undefined;
let extra: Record<string, unknown> = {};
if (mode === "every") stepTo(ticks, true);
else if (mode === "capture") stepTo(ticks, false);
else if (mode === "restore") {
  stepTo(forkAt, true);
  const fork = probe.snapshot() as Snap;
  stepTo(restoreAt, true);
  const saved = probe.snapshot() as Snap;
  stepTo(ticks, true);
  const a = shot();
  first = a.sha;
  if (out) writeFileSync(out.replace(/\.png$/, ".first.png"), a.png);
  // Another branch from the fork, past the snapshot, without the inputs.
  probe.restore(fork.id);
  const kept = new Map(inputs);
  inputs.clear();
  stepTo(ticks, true);
  for (const [k, v] of kept) inputs.set(k, v);
  probe.restore(saved.id);
  stepTo(ticks, true);
} else if (mode === "synctest") {
  stepTo(Math.max(0, restoreAt - rollback), true);
  const ring: Snap[] = [];
  const mismatches: string[] = [];
  let checked = 0;
  while (tick() < ticks) {
    ring.push(probe.snapshot() as Snap);
    if (ring.length > rollback) ring.shift();
    stepTo(tick() + 1, true);
    const now = probe.snapshot() as Snap;
    if (ring.length === rollback && now.tick >= restoreAt) {
      // Back `rollback` ticks and forward again: GGPO's SyncTest.
      probe.restore(ring[0]!.id);
      stepTo(now.tick, true);
      const again = probe.snapshot() as Snap;
      checked++;
      if (again.hash !== now.hash || again.presentation !== now.presentation) {
        mismatches.push(`tick ${now.tick}: world ${now.hash} then ${again.hash}, presentation ${now.presentation} then ${again.presentation}`);
      }
    }
  }
  extra = { checked, mismatches };
} else if (mode === "ticks") {
  const each: Array<{ tick: number; hash: string; presentation: string }> = [];
  while (tick() < ticks) {
    stepTo(tick() + 1, true);
    const { tick: t, hash, presentation } = probe.snapshot() as Snap;
    each.push({ tick: t, hash, presentation });
  }
  // og.motion's schedule at the end (dump_motion): the batches' ticks, each agent's frames.
  probe.commands("0:dump_motion:motion.json");
  const f = engine.frame(0);
  if (!f.ok) throw new Error(f.reason);
  const dump = engine.takeOutputs().find((o) => o.action === "dump_motion");
  extra = { ticks: each, motion: dump ? JSON.parse(dump.data) : null };
} else if (mode === "display") {
  // The display's own frames, not captures: drawn on every tick from
  // --fork-at to --ticks, then again after another branch and a restore
  // of the fork's snapshot; each tick's frame must be the first pass's.
  stepTo(forkAt, true);
  const fork = probe.snapshot() as Snap;
  const pass = (): Uint8Array[] => {
    const frames: Uint8Array[] = [];
    while (tick() < ticks) {
      stepTo(tick() + 1, true);
      frames.push(Uint8Array.from(native().og_node_render_frame(engine.id)));
    }
    return frames;
  };
  const a = pass();
  if (argv.includes("--frames")) {
    const dir = opt("--frames");
    mkdirSync(dir, { recursive: true });
    a.forEach((f, i) => writeFileSync(join(dir, `${forkAt + i + 1}.rgba`), f));
  }
  probe.restore(fork.id);
  const kept = new Map(inputs);
  inputs.clear();
  stepTo(Math.min(ticks, forkAt + 24), true);
  for (const [k, v] of kept) inputs.set(k, v);
  probe.restore(fork.id);
  const b = pass();
  // Each tick whose frame differs: the pixels that differ and the largest channel difference.
  const mismatches = a.flatMap((x, i) => {
    const y = b[i]!;
    let pixels = 0;
    let max = 0;
    for (let p = 0; p < x.length; p += 4) {
      let d = 0;
      for (let c = 0; c < 4; c++) d = Math.max(d, Math.abs(x[p + c]! - y[p + c]!));
      if (d > 0) pixels++;
      max = Math.max(max, d);
    }
    return pixels === 0 && x.length === y.length ? [] : [`tick ${forkAt + i + 1}: ${pixels} px, max ${max}`];
  });
  extra = { checked: a.length, mismatches };
} else throw new Error(`--mode is every, capture, restore, synctest, ticks or display, not ${mode}`);
const frame = shot();
if (out) writeFileSync(out, frame.png);
const presentation = (probe.snapshot() as Snap).presentation;
process.stdout.write(JSON.stringify({ mode, tick: tick(), hash: engine.hash(), presentation, png: frame.sha, ...(first ? { first } : {}), ...extra }) + "\n");
engine.destroy();
