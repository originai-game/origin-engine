// The project template end to end on the native host: `create` scaffolds it,
// `og test` cooks its assets (a behaviour and a navmesh among them) and runs
// its playtests through `og run --headless --commands` (the smoke test and
// the guard on og.ai and og.nav), and the run leaves the frame and the dumps
// the smoke test checked. The scaffold stays in target/evidence/template/ for inspection.
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(
    cmd: string[],
    o: { cwd: string; env?: Record<string, string | undefined> },
  ): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const evidence = join(repo, "target", "evidence", "template");
const game = join(evidence, "smoke-game");

const run = (cmd: string[]) => {
  const r = Bun.spawnSync(cmd, { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const decode = (b: Uint8Array) => new TextDecoder().decode(b);
  return { code: r.exitCode, out: decode(r.stdout) + decode(r.stderr) };
};

test("a scaffolded game cooks, passes its playtest and leaves the frame and dumps", () => {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = run(["bun", join(repo, "packages", "create", "bin", "create.ts"), game, "--name", "Smoke Game"]);
  expect(created.code).toBe(0);
  expect(existsSync(join(game, "og-out"))).toBeFalse();

  const tested = run([ogBin(), "test", game]);
  if (tested.code !== 0) throw new Error(tested.out);
  expect(tested.out).toMatch(/PASS smoke \(8 checks/);
  expect(tested.out).toMatch(/PASS guard \(16 checks/);
  expect(tested.out).toContain("2 suite(s) passed");

  const out = join(game, "og-out", "test", "smoke");
  const png = readFileSync(join(out, "smoke.png"));
  expect([...png.subarray(1, 4)]).toEqual([0x50, 0x4e, 0x47]);
  const world = JSON.parse(readFileSync(join(out, "world.json"), "utf8")) as {
    entities: number;
    systems: Array<{ name: string; matched: number | null }>;
  };
  expect(world.entities).toBeGreaterThan(3);
  expect(world.systems.find((s) => s.name === "Rotate")?.matched).toBe(1);
  const ui = JSON.parse(readFileSync(join(out, "ui.json"), "utf8")) as { elements: Array<{ id: string; text: string }> };
  expect(ui.elements.find((e) => e.id === "Start")?.text).toBe("Started");
}, 600_000);
