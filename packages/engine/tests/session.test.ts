import { expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { bundleGame } from "../src/node/build.ts";
import { run } from "../src/node/run.ts";

declare const Bun: {
  spawnSync(args: string[], options: { stdout: "pipe"; stderr: "pipe" }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

test("fresh runs isolate module state, world mutations and failed setup", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-session-test-"));
  const source = join(dir, "main.ts");
  const entry = join(dir, "game.js");
  const runner = resolve(import.meta.dir!, "../src/node/run.ts");
  try {
    writeFileSync(source, `import { og } from "@origingame/engine";
let starts = 0;
export default og.game({
  start(world) { starts++; for (let i=0; i<starts; i++) world.spawn({name:"initial-"+i}); },
  commands: { mutate(world) { starts += 20; world.spawn({name:"contamination"}); } },
  facts() { return { starts }; }
});`);
    const bundled = await bundleGame(source, entry, "native");
    if (!bundled.ok) throw new Error(bundled.reason);
    const args = (script: string) => ["--entry", entry, "--no-render", "--seed", "83", "--clock", "1767225600000", "--commands", script, "--json", "--out", dir];
    const script = "0.2:dump_world:world.json;0.3:quit";
    const cold = Bun.spawnSync(["bun", runner, ...args(script)], { stdout: "pipe", stderr: "pipe" });
    expect(cold.exitCode).toBe(0);
    const signature = (stdout: string) => {
      const value = JSON.parse(stdout.trim().split("\n").pop()!);
      return {
        hash: value.hash, tick: value.tick, entities: value.entities, facts: value.facts, audio: value.audio,
        assertions: { violations: value.assertions.violations, sites: value.assertions.sites, uncovered: value.assertions.uncovered },
      };
    };
    const expected = signature(new TextDecoder().decode(cold.stdout));
    expect(expected.entities).toBe(1);
    expect(expected.facts.starts).toBe(1);
    const invoke = async (argv: string[]) => {
      let stdout = "";
      let stderr = "";
      const code = await run(argv, { stdout: { write: (s) => { stdout += s; } }, stderr: { write: (s) => { stderr += s; } } });
      return { code, stdout, stderr };
    };
    for (let i = 0; i < 3; i++) {
      const actual = await invoke(args(script));
      expect(actual.code).toBe(0);
      expect(signature(actual.stdout)).toEqual(expected);
      const mutation = await invoke(args("0.1:mutate;0.3:quit"));
      expect(mutation.code).toBe(0);
      expect(signature(mutation.stdout).entities).toBe(2);
      expect(signature(mutation.stdout).facts.starts).toBe(21);
      const failed = await invoke([...args(script), "--setup", "not_a_command"]);
      expect(failed.code).toBe(2);
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 30_000);
