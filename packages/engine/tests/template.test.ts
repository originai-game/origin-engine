import { gpuTest } from "./support/gpu.ts";
// Creation and the native host use the same empty skeleton. Feature
// behaviour belongs in tests/fixtures, not in the scaffold.
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

gpuTest("a scaffolded game cooks and reaches quit without sample gameplay", () => {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = run(["bun", join(repo, "packages", "create", "bin", "create.ts"), game, "--name", "Smoke Game"]);
  expect(created.code).toBe(0);
  expect(existsSync(join(game, "og-out"))).toBeFalse();

  const tested = run([ogBin(), "test", game]);
  if (tested.code !== 0) throw new Error(tested.out);
  expect(tested.out).toContain("PASS smoke");
  expect(readFileSync(join(game, "src/main.ts"), "utf8")).toContain("export default og.game({});");
  expect(existsSync(join(game, "assets/scenes"))).toBeFalse();
  const out = join(evidence, "empty-world");
  const empty = run([ogBin(), "run", game, "--headless", "--no-render", "--out", out, "--commands", "0.1:dump_world:world.json;0.2:quit"]);
  expect(empty.code, empty.out).toBe(0);
  expect(JSON.parse(readFileSync(join(out, "world.json"), "utf8")).entities).toBe(0);
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
  const at = lines.findIndex((l) => l.startsWith("import "));
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
