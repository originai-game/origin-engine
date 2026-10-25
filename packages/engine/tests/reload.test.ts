// A hot reload through `run.ts --watch` (what `og dev` drives) takes the
// changed bundle: the re-imported module declares again, the probe's reload
// swaps it in and a changed component layout migrates by name. A re-import
// that answered the module already evaluated would declare nothing and the
// reload would be refused ("never declared").
import { expect, test } from "bun:test";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// The gate's path-aware lane skips this file: it spawns og processes for
// minutes (docs/ci.md §Lanes). `scripts/check.sh --all`, CI and a plain
// `bun run test` run it (OG_GATE_ALL, or no OG_GATE at all).
const SLOW = process.env.OG_GATE === "1" && !process.env.OG_GATE_ALL;

declare const Bun: {
  spawn(cmd: string[], o: { cwd: string; stdout: "pipe"; stderr: "pipe" }): { stdout: ReadableStream<Uint8Array>; exited: Promise<number>; kill(): void };
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
  test.skipIf(SLOW)(`a watched bundle reloads and migrates a changed layout (render=${render})`, async () => {
    const dir = join(pkg, "tests", `.reload-${process.pid}-${render}`);
    mkdirSync(dir, { recursive: true });
    const entry = join(dir, "game.ts");
    writeFileSync(entry, game("a: f32(1)"));
    const child = Bun.spawn(["bun", "src/node/run.ts", "--entry", entry, "--watch", entry, "--ticks", "240", "--realtime", ...(render ? ["--size", "160x90"] : ["--no-render"]), "--json"], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
    // The changed bundle is written every half second until the run ends,
    // not once at a fixed delay: run.ts starts watching only once its engine
    // (and renderer) are up, which took longer than 1.5 s with a renderer on
    // CI's two vCPU of Lavapipe, and a write before that is never seen. Its
    // 240 paced ticks start after the watch, so a write lands in them.
    // Rewriting the same bundle reloads the same layout again.
    const writer = setInterval(() => writeFileSync(entry, game("a: f32(1), b: f32(2)")), 500);
    const out = await new Response(child.stdout).text();
    clearInterval(writer);
    expect(await child.exited).toBe(0);
    rmSync(dir, { recursive: true, force: true });
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
