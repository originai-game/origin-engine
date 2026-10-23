// The project template end to end on the native host: `create` scaffolds it,
// `og test` cooks its assets (a behaviour and a navmesh among them) and runs
// its playtests through `og run --headless --commands` (the smoke test and
// the guard on og.ai and og.nav), and the run leaves the frame and the dumps
// the smoke test checked. The scaffold stays in target/evidence/template/ for inspection.
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
  expect(tested.out).toMatch(/PASS smoke \(9 checks/);
  expect(tested.out).toMatch(/PASS guard \(16 checks/);
  expect(tested.out).toMatch(/PASS intro \(17 checks/);
  expect(tested.out).toContain("3 suite(s) passed");

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

// A scaffolded game is type-checked by og (og check, tsrs): `og check` passes
// on it as scaffolded, a type error is a value in `--json` (code 2322 at
// its line), and og run, og test and og build stop on it before bundling.
test("a scaffolded game type-checks clean, and a type error stops og before it bundles", () => {
  const dir = join(repo, "target", "evidence", "template-check");
  const checked = join(dir, "game");
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  expect(run(["bun", join(repo, "packages", "create", "bin", "create.ts"), checked, "--name", "Checked Game"]).code).toBe(0);

  const clean = run([ogBin(), "check", checked, "--json"]);
  if (clean.code !== 0) throw new Error(clean.out);
  expect(JSON.parse(clean.out.trim().split("\n").at(-1)!)).toMatchObject({ event: "check", ok: true, diagnostics: [] });

  const main = join(checked, "src", "main.ts");
  const source = readFileSync(main, "utf8");
  const lines = source.split("\n");
  const at = lines.findIndex((l) => l.startsWith("export const Velocity"));
  lines.splice(at + 1, 0, 'export const speed: number = "fast";');
  writeFileSync(main, lines.join("\n"));
  const bad = run([ogBin(), "check", checked, "--json"]);
  expect(bad.code).toBe(1);
  const event = JSON.parse(bad.out.trim().split("\n").at(-1)!) as { ok: boolean; diagnostics: Array<{ file: string; line: number; code: number }> };
  expect(event.ok).toBeFalse();
  expect(event.diagnostics.map((d) => [d.file, d.line, d.code])).toEqual([["src/main.ts", at + 2, 2322]]);
  for (const cmd of [["run", "--headless", "--ticks", "1"], ["test"], ["build"]]) {
    const r = run([ogBin(), ...cmd, checked]);
    expect(r.code).toBe(1);
    expect(r.out).toContain("1 type error(s); fix them or pass --no-check");
  }
  expect(existsSync(join(checked, ".og", "game.native.js"))).toBeFalse();
  expect(existsSync(join(checked, "dist"))).toBeFalse();
}, 120_000);
