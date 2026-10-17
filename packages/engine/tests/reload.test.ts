// A hot reload through `run.ts --watch` (what `og dev` drives) takes the
// changed bundle: the re-imported module declares again, the probe's reload
// swaps it in and a changed component layout migrates by name. A re-import
// that answered the module already evaluated would declare nothing and the
// reload would be refused ("never declared").
import { expect, test } from "bun:test";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

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

test("a watched bundle reloads with its new declarations and migrates a changed layout", async () => {
  const dir = join(pkg, "tests", `.reload-${process.pid}`);
  mkdirSync(dir, { recursive: true });
  const entry = join(dir, "game.ts");
  writeFileSync(entry, game("a: f32(1)"));
  const child = Bun.spawn(["bun", "src/node/run.ts", "--entry", entry, "--watch", entry, "--ticks", "240", "--realtime", "--no-render", "--json"], { cwd: pkg, stdout: "pipe", stderr: "pipe" });
  setTimeout(() => writeFileSync(entry, game("a: f32(1), b: f32(2)")), 1500);
  const out = await new Response(child.stdout).text();
  await child.exited;
  rmSync(dir, { recursive: true, force: true });
  const reloads = out.split("\n").filter((l) => l.startsWith('{"event":"reload"')).map((l) => JSON.parse(l));
  expect(reloads.length).toBeGreaterThan(0);
  const last = reloads.at(-1)!;
  expect(last.ok).toBeTrue();
}, 60_000);
