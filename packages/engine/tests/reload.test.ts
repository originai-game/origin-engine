// A hot reload through `run.ts --watch` (what `og dev` drives) takes the
// changed bundle: the re-imported module declares again, the probe's reload
// swaps it in and a changed component layout migrates by name. A re-import
// that answered the module already evaluated would declare nothing and the
// reload would be refused ("never declared").
import { expect, test } from "bun:test";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gpuUnavailable } from "./support/gpu.ts";
import { ogBin } from "./support/og-bin.ts";

// The gate's path-aware lane skips this file: it spawns og processes for
// minutes (docs/ci.md §Lanes). `scripts/check.sh --all`, CI and a plain
// `bun run test` run it (OG_GATE_ALL, or no OG_GATE at all).
const SLOW = process.env.OG_GATE === "1" && !process.env.OG_GATE_ALL;

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): { pid: number; stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>; kill(signal?: string): void };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

const game = (fields: string) => `
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
const Score = defineResource("RlScore", { n: u32() });
const Tag = defineComponent("RlTag", { ${fields} });
const Count = defineSystem("RlCount", { reads: [Tag], writes: [Score] }, (_q, _t, world) => {
  const s = world.resource(Score);
  if (s.ok) s.value.n += 1;
});
export default og.game({ resources: [Score], systems: [Count], start(world) {
  const e = world.spawn({ name: "tagged" });
  if (e.ok) world.insert(e.value, Tag);
} });
`;

for (const render of [false, true]) {
  test.skipIf(SLOW || (render && gpuUnavailable))(`a watched bundle reloads and migrates a changed layout (render=${render})`, async () => {
    const dir = join(pkg, "tests", `.reload-${process.pid}-${render}`);
    mkdirSync(dir, { recursive: true });
    const entry = join(dir, "game.ts");
    writeFileSync(entry, game("a: f32(1)"));
    const child = Bun.spawn(["bun", "src/node/run.ts", "--entry", entry, "--watch", entry, "--port", "0", "--realtime", ...(render ? ["--size", "160x90"] : ["--no-render"]), "--json"], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
    const stderr = new Response(child.stderr).text();
    const reader = child.stdout.getReader();
    let out = "";
    const until = async (marker: string) => {
      while (!out.includes(marker)) {
        const { value, done } = await reader.read();
        if (done) throw new Error(`run ended before ${marker}: ${out}\n${await stderr}`);
        out += new TextDecoder().decode(value);
      }
    };
    try {
      await until("probe: ws://");
      // The watcher is ready. One edit, then wait for its outcome rather
      // than ending at a fixed tick while its import may still be in flight.
      writeFileSync(entry, game("a: f32(1), b: f32(2)"));
      await until('{"event":"reload"');
      const port = /probe: ws:\/\/127\.0\.0\.1:(\d+)/.exec(out)![1]!;
      const stop = Bun.spawn([ogBin(), "probe", "commands", '{"script":"0:quit"}', "--port", port], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
      expect(await stop.exited).toBe(0);
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        out += new TextDecoder().decode(value);
      }
      expect(await child.exited).toBe(0);
      expect(await stderr).not.toContain("WARN reload:");
    } finally {
      child.kill();
      await child.exited;
      rmSync(dir, { recursive: true, force: true });
    }
    const reloads = out.split("\n").filter((l) => l.startsWith('{"event":"reload"')).map((l) => JSON.parse(l));
    expect(reloads.length).toBeGreaterThan(0);
    const last = reloads.at(-1)!;
    expect(last.ok).toBeTrue();
    expect(last.rendered).toBe(render);
    for (const stage of ["importMs", "reloadMs", "frameMs", "gpuWaitMs"]) {
      expect(Number.isFinite(last.timings[stage])).toBeTrue();
      expect(last.timings[stage]).toBeGreaterThanOrEqual(0);
    }
  }, 60_000);
}

// `og dev --engine`'s hot restart: a run with --handoff writes its world on
// SIGUSR2 and exits 0, and a new process (another engine build, here another
// layout of RlTag) restores it with --restore and goes on from that tick,
// the changed layout migrated by name. A restore that started over, or one
// refused for the new field, would report another tick or ok false.
test.skipIf(SLOW)("a handed-off world restores in a new process with a changed layout", async () => {
  const dir = join(pkg, "tests", `.handoff-${process.pid}`);
  mkdirSync(dir, { recursive: true });
  const entry = join(dir, "game.ts");
  const save = join(dir, "world.save");
  writeFileSync(entry, game("a: f32(1)"));
  const first = Bun.spawn(["bun", "src/node/run.ts", "--entry", entry, "--no-render", "--realtime", "--port", "0", "--handoff", save], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
  const reader = first.stdout.getReader();
  let out = "";
  while (!out.includes("probe: ws://")) {
    const { value, done } = await reader.read();
    if (done) break;
    out += new TextDecoder().decode(value);
  }
  // Some ticks run first, so a restore that started over shows.
  await new Promise((r) => setTimeout(r, 500));
  // process.kill, not first.kill("SIGUSR2"): Bun 1.4.2's Subprocess.kill
  // sends a signal name by Linux's number, and 12 is SIGSYS on macOS
  // (oven-sh/bun#35296, fixed in Bun 1.4.3).
  (process as unknown as { kill(pid: number, signal: string): void }).kill(first.pid, "SIGUSR2");
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    out += new TextDecoder().decode(value);
  }
  expect(await first.exited).toBe(0);
  const handoff = out.split("\n").filter((l) => l.startsWith('{"event":"handoff"')).map((l) => JSON.parse(l))[0];
  expect(handoff.tick).toBeGreaterThan(0);

  writeFileSync(entry, game("a: f32(1), b: f32(2)"));
  const second = Bun.spawn(["bun", "src/node/run.ts", "--entry", entry, "--no-render", "--restore", save, "--commands", "0.1:quit"], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
  const text = await new Response(second.stdout).text();
  expect(await second.exited).toBe(0);
  rmSync(dir, { recursive: true, force: true });
  const restored = text.split("\n").filter((l) => l.startsWith('{"event":"restored"')).map((l) => JSON.parse(l))[0];
  expect(restored).toEqual({ event: "restored", ok: true, tick: handoff.tick });
}, 60_000);
