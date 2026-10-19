// The Phase 2 core round on both hosts (tests/support/core-round.ts runs each
// case in its own process: the native addon, a dev build; the wasm module, a
// release build; the publish wasm).
//
// Ways it could fail, and the test that catches each:
// - a fork from og.Rng declared only in `reads` answers NaN, moves the main
//   stream, or differs between hosts → "og.Rng in reads forks …"
// - an undeclared resource read, resource write, og.Rng draw, event read or
//   emit inside a system is a silent refusal in a dev or release build, or
//   names neither the system nor the record → "an undeclared access throws …"
// - a publish build crashes on one instead, logs it every tick, or not at
//   all → "a publish build refuses an undeclared access and logs it once …"
// - og.platform.now is 0, reads the clock inside the tick (so it does not
//   follow the ticks under a fixed clock), or an answer carries no time, or
//   the hosts disagree → "og.platform.now and the answers' time …"
// - an input event loses its time within the tick (the host's stamp or the
//   command step's), keeps it on the edges the tick's end raises, is not
//   clamped, or the hosts disagree → "input events carry their time …"
// - og.audio.outputLatency is not 0 headless, or the web Worker's report of
//   an AudioContext's latency (and its prime) does not reach it, or a later
//   report does not move it → "og.audio.outputLatency …"
// - `og test` forgets the store between a playtest's processes when asked to
//   keep it, keeps it when not, ignores a step's clock, or never runs the
//   `then` steps → "og test keeps the local store across a playtest's
//   processes …" (tests/fixtures/relaunch)
import { expect, test } from "bun:test";
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { ogBin } from "./support/og-bin.ts";
import { fileURLToPath } from "node:url";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

function run<T = Record<string, unknown>>(host: string, which: string): T {
  const r = Bun.spawnSync(["bun", "tests/support/core-round.ts", "--host", host, "--case", which], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} ${which}: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as T;
}

type Rng = { advanced: { ok: boolean }; repro: Record<string, number>; rng: { seed: number; state: number }; hash: string };

test("og.Rng in reads forks a deterministic stream without moving the main one, alike on both hosts", () => {
  const node = run<Rng>("node", "rng");
  const web = run<Rng>("web", "rng");
  for (const r of [node, web]) {
    expect(r.advanced.ok).toBeTrue();
    // The archetypes round's reproduction: the fork from reads answered NaN, from writes 0.6868….
    expect(r.repro.readsDraw).toBe(r.repro.writesDraw);
    expect(r.repro.readsDraw).toBeCloseTo(0.6868846963625401, 15);
    expect(r.repro.readsSeed).toBe(1);
    // Forking reads the seed only: the main stream did not move.
    expect(r.repro.readsStateAfter).toBe(r.repro.readsStateBefore);
    expect(r.rng.state).toBe(r.repro.readsStateBefore);
  }
  expect(web.repro).toEqual(node.repro);
  expect(web.hash).toBe(node.hash);
}, 120_000);

type Access = { ok: boolean; reason: string; tick: number; seen: { tries: number; refused: number }; shared: number; log: string[] };
type Undeclared = Record<"read" | "write" | "draw" | "events" | "emit", Access>;

const NAMES = {
  read: ["CrRead", "CrShared", "reads or writes"],
  write: ["CrWrite", "CrShared", "writes"],
  draw: ["CrDraw", "og.Rng", "writes"],
  events: ["CrEvents", "og.KeyDown", "reads or writes"],
  emit: ["CrEmit", "og.KeyUp", "writes"],
} as const;

test("an undeclared access throws with the system and the record in dev and release builds, failing the tick", () => {
  for (const host of ["node", "web"]) {
    const r = run<Undeclared>(host, "undeclared");
    for (const [kind, [system, record, list]] of Object.entries(NAMES)) {
      const a = r[kind as keyof Undeclared];
      expect(a.ok).toBeFalse();
      expect(a.tick).toBe(0);
      expect(a.reason).toContain(`system '${system}'`);
      expect(a.reason).toContain(`did not declare '${record}' among its ${list}`);
    }
    expect(r.write.shared).toBe(7);
  }
}, 120_000);

test("a publish build refuses an undeclared access and logs it once per system, and the game keeps running", () => {
  const r = run<Undeclared>("publish", "undeclared");
  for (const [kind, [system, record, list]] of Object.entries(NAMES)) {
    const a = r[kind as keyof Undeclared];
    expect(a.ok).toBeTrue();
    expect(a.tick).toBe(5);
    expect(a.seen.tries).toBe(5);
    const lines = a.log.filter((l) => l.includes(`system '${system}'`));
    expect(lines.length).toBe(1);
    expect(lines[0]).toContain(`did not declare '${record}' among its ${list}`);
  }
  expect(r.read.seen.refused).toBe(5);
  expect(r.draw.seen.refused).toBe(5);
  expect(r.emit.seen.refused).toBe(5);
  expect(r.write.shared).toBe(7);
}, 120_000);

type Time = {
  T: number;
  fixed: { startNow: number; answerTick: number; answerTime: number; nowThen: number; nowAt5: number };
  clock: string;
  nowAfter: number;
  systemClock: string;
  systemSkewMs: number;
};

test("og.platform.now and the answers' time follow the host's clock, as host input, alike on both hosts", () => {
  const node = run<Time>("node", "time");
  const web = run<Time>("web", "time");
  for (const r of [node, web]) {
    const tick = 1000 / 60;
    expect(r.clock).toBe("fixed");
    // Told when the platform attached: tick 0 reads the clock's start.
    expect(r.fixed.startNow).toBe(r.T);
    // The call on tick 1 is answered between ticks 1 and 2 and applied at tick 2's start, with the host's time then.
    expect(r.fixed.answerTick).toBe(2);
    expect(r.fixed.answerTime).toBeCloseTo(r.T + 2 * tick, 6);
    expect(r.fixed.nowThen).toBeCloseTo(r.fixed.answerTime, 6);
    // Between answers now() moves with the ticks.
    expect(r.fixed.nowAt5).toBeCloseTo(r.T + 5 * tick, 6);
    expect(r.nowAfter).toBeCloseTo(r.T + 8 * tick, 6);
    // Without a fixed clock it is the machine's.
    expect(r.systemClock).toBe("system");
    expect(r.systemSkewMs).toBeLessThan(5000);
  }
  expect(web.fixed).toEqual(node.fixed);
}, 120_000);

test("og test keeps the local store across a playtest's processes with store: keep, and each its own clock", () => {
  const repo = join(pkg, "..", "..");
  const game = join(repo, "target", "evidence", "relaunch");
  rmSync(game, { recursive: true, force: true });
  mkdirSync(dirname(game), { recursive: true });
  cpSync(join(pkg, "tests", "fixtures", "relaunch"), game, { recursive: true });
  const r = Bun.spawnSync([ogBin(), "test", game], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const out = new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
  if (r.exitCode !== 0) throw new Error(out);
  expect(out).toMatch(/PASS away \(7 checks/);
  expect(out).toMatch(/PASS fresh \(2 checks/);
  expect(out).toContain("2 suite(s) passed");
}, 300_000);

type Seen = { seen: Array<{ tick: number; kind: string; key?: string; id?: number; at: number }>; hash: string };

test("input events carry their time within the tick, from the host's stamp and from the command channel, alike on both hosts", () => {
  const node = run<Seen>("node", "input");
  const web = run<Seen>("web", "input");
  expect(node.seen).toEqual([
    { tick: 0, kind: "down", key: "KeyA", at: 0.25 },
    { tick: 1, kind: "up", key: "KeyA", at: -0.5 },
    { tick: 1, kind: "pointer", id: 2, at: 0.75 },
    // Clamped to a tick; an event with no time is at the tick's start.
    { tick: 2, kind: "down", key: "KeyB", at: 1 },
    { tick: 2, kind: "down", key: "KeyC", at: 0 },
    // A press's release comes from the tick's end, with no time of its own.
    { tick: 3, kind: "up", key: "KeyB", at: 0 },
    // Command steps: their time less the tick they land on.
    { tick: 10, kind: "down", key: "KeyD", at: 0 },
    { tick: 10, kind: "down", key: "KeyE", at: 0.498 },
    { tick: 12, kind: "down", key: "KeyF", at: -0.5 },
  ]);
  expect(web.seen).toEqual(node.seen);
  // The stamp is world state: both hosts hash the same world.
  expect(web.hash).toBe(node.hash);
}, 120_000);

type Latency = { headless: number; outside: number; reported?: number; expected?: number; moved?: number; expectedMoved?: number };

test("og.audio.outputLatency is 0 headless and the web host's report of its AudioContext reaches it between ticks", () => {
  for (const host of ["node", "web"]) {
    const r = run<Latency>(host, "latency");
    expect(r.headless).toBe(0);
    expect(r.outside).toBe(0);
  }
  const web = run<Latency>("web", "latency");
  expect(web.reported).toBeCloseTo(web.expected!, 9);
  expect(web.expected).toBeCloseTo(0.012 + 2048 / 48000, 9);
  expect(web.moved).toBeCloseTo(web.expectedMoved!, 9);
}, 120_000);
