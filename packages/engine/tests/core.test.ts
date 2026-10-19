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
import { expect, test } from "bun:test";
import { dirname, join } from "node:path";
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
