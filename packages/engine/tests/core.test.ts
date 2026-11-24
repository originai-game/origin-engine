// The Phase 2 core round on both hosts (tests/support/core-round.ts runs each
// case in its own process: the native addon, a dev build; the wasm module, a
// release build; the publish wasm).
//
// Ways it could fail, and the test that catches each:
// - a fork from og.Rng declared only in `reads` answers NaN, moves the main
//   stream, or differs between hosts → "og.Rng in reads forks …"
// - og.procgen or og.Rng's picks leave og-core's pinned outputs, or the
//   hosts disagree → "og.procgen and og.Rng's picks …"
// - og.audio.createSound refuses a whole sound, or an og.AudioEvent's pick
//   does not follow the world's seed → "og.audio.createSound …"
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// The functional lane may run with a web module older than the native addon
// (scripts/check.sh bun-functional): the tests that compare the two hosts
// or load the web module skip there and run in the full lane.
const STALE_WASM = process.env.OG_WASM_STALE === "1";

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

test.skipIf(STALE_WASM)("og.Rng in reads forks a deterministic stream without moving the main one, alike on both hosts", () => {
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

type Procgen = { hashes: Record<string, string | number>; noise: number; refused: { ok: boolean; reason: string }; hash: string };

/** The shared procgen pin (tests/fixtures/procgen-pin.json): the scenario and the cross-host hashes, also og-core's test's. */
const procgenPin = JSON.parse(
  readFileSync(join(pkg, "..", "..", "tests", "fixtures", "procgen-pin.json"), "utf8"),
) as { pinned: Record<string, string> };

test("og.procgen and og.Rng's picks give og-core's pinned outputs, bit for bit, through the API", () => {
  const node = run<Procgen>("node", "procgen");
  // og-core's procgen::tests::every_generator_gives_its_pinned_output_for_a_seed:
  // the floats survive the JSON wire exactly, so the hashes are the same.
  expect(node.hashes).toMatchObject({
    ...procgenPin.pinned,
    // og.Rng's fork "picks" of a world seeded 42: equal across runs and hosts.
    weighted: "d9faff60662c6855",
    permutation: "496debfddbe40345",
    mainWeighted: 2,
  });
  expect(node.noise).toBeGreaterThan(-1);
  expect(node.noise).toBeLessThan(1);
  expect(node.refused.ok).toBeFalse();
  expect(node.refused.reason).toContain("0 x 32");
  if (STALE_WASM) return;
  const web = run<Procgen>("web", "procgen");
  expect(web.hashes).toEqual(node.hashes);
  expect(web.noise).toBe(node.noise);
  expect(web.refused).toEqual(node.refused);
  expect(web.hash).toBe(node.hash);
}, 120_000);

type Events = { info: unknown; refused: { ok: boolean; reason: string }; picks: number[]; hash: string };

function audioEvents(host: string, seed: number): Events {
  const r = Bun.spawnSync(["bun", "tests/support/core-round.ts", "--host", host, "--case", "audioEvents", "--seed", String(seed)], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} audioEvents: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as Events;
}

test("og.audio.createSound makes sounds from TypeScript and an og.AudioEvent picks among them by the world's seed", () => {
  const a = audioEvents("node", 5);
  const b = audioEvents("node", 5);
  const c = audioEvents("node", 6);
  expect(a.info).toMatchObject({ ok: true, value: { rate: 48000, channels: 1, frames: 2400, chunks: 0 } });
  expect(a.refused.ok).toBeFalse();
  expect(a.refused.reason).toContain("whole frames");
  // A trigger every other tick and a 0.05 s cooldown (3 ticks): every fourth tick plays.
  expect(a.picks.length).toBe(30);
  expect(new Set(a.picks).size).toBe(4);
  expect(b.picks).toEqual(a.picks);
  expect(b.hash).toBe(a.hash);
  expect(c.picks).not.toEqual(a.picks);
}, 120_000);
