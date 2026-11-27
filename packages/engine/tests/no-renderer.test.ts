// Simulation does not need a GPU; a test that requests pixels does.
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], options: { cwd: string; env: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const evidence = join(repo, "target", "evidence", "no-renderer");
const game = join(evidence, "game");

const run = (cmd: string[], renderer: boolean) => {
  const env = { ...process.env, OG_BIN: ogBin(), OG_ENGINE_PACKAGE: pkg, OG_RENDERER: renderer ? process.env.OG_RENDERER : "none" };
  const r = Bun.spawnSync(cmd, { cwd: repo, env });
  const stdout = new TextDecoder().decode(r.stdout);
  return { code: r.exitCode, stdout, out: stdout + new TextDecoder().decode(r.stderr) };
};

interface Result {
  playtest: string;
  ok: boolean;
  status: string;
  renderer: string;
  adapterError: string | null;
  checks: number;
  frameChecks: number;
  unchecked: string[];
  failures: string[];
  tier: string;
  hash: string;
  hashes: string[];
  failureCaptures: { tick: number | null; world: string | null; snapshot: string | null; frame: { status: "captured"; path: string } | { status: "unavailable"; reason: string }; files: { path: string; tick: number | null; what: string }[] }[];
}

const result = (stdout: string): Result => JSON.parse(stdout.split("\n").find((l) => l.startsWith('{"playtest"'))!);

const SURFACE = `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("a surface check needs a GPU without changing simulation", async () => {
  const run = await runPlaytest({ size: "640x480", runs: 2, commands: "0.3:dump_ui:ui.json;0.4:screenshot:surface.png;0.5:dump_world:world.json;0.6:quit" });
  expect(run.warnings, "no warnings").toEqual([]);
  expect(run.quit, "reached quit").toBe(true);
  expect(run.hashes[1], "repeatable world").toBe(run.hashes[0]);
  const ui = run.json<{ surface: { width: number; height: number } }>("ui.json");
  expect([ui.surface.width, ui.surface.height], "requested surface").toEqual([640, 480]);
  run.frame("surface.png", () => {
    const png = run.png("surface.png");
    expect([png.width, png.height], "rendered surface").toEqual([640, 480]);
  });
});
`;

function prepare() {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = run(["bun", join(repo, "packages/create/bin/create.ts"), game, "--name", "No Renderer"], true);
  if (created.code !== 0) throw new Error(created.out);
  writeFileSync(join(game, "src/main.ts"), `import { og } from "@origingame/engine"; export default og.game({});`);
  writeFileSync(join(game, "tests/surface.og.ts"), SURFACE);
  return [ogBin(), "test", game, "--filter", "surface.og.ts", "--json", "--no-check"];
}

test("missing GPU reports unchecked frames, while simulation checks still run and fail honestly", () => {
  const command = prepare();
  const none = run(command, false);
  expect(none.code).toBe(0);
  const blind = result(none.stdout);
  expect(blind.ok).toBeTrue();
  expect(blind.status).toBe("unchecked");
  expect(blind.renderer).toBe("none");
  expect(blind.adapterError).toContain("no GPU adapter");
  expect(blind.tier).toBe("none");
  // Four expects, the frame's report that it went unchecked, and runPlaytest's own
  // for its screenshot left unchecked (a script check, never a pass).
  expect(blind.checks).toBe(6);
  expect(blind.frameChecks).toBe(0);
  expect(blind.unchecked).toEqual(["surface.png"]);
  expect(blind.failures).toEqual([]);
  const aggregate = JSON.parse(none.stdout.split("\n").find((l) => l.startsWith("{") && JSON.parse(l).event === "test")!);
  expect(aggregate).toMatchObject({ ok: true, passed: 0, failed: 0, uncheckedSuites: 1, unchecked: 1, renderer: "none" });
  expect(none.out).toContain("WARNING missing GPU");
  expect(none.out).not.toContain("unexpected");
  const out = join(game, "og-out/test/surface/run-1");
  expect(existsSync(join(out, "surface.png"))).toBeFalse();

  const strict = run([...command, "--require-checked"], false);
  expect(strict.code).toBe(1);
  const strictAggregate = JSON.parse(strict.stdout.split("\n").find((l) => l.startsWith("{") && JSON.parse(l).event === "test")!);
  expect(strictAggregate).toMatchObject({ ok: false, failed: 0, uncheckedSuites: 1, uncheckedCount: 1, unchecked: 1 });
  const runCommand = [ogBin(), "run", game, "--headless", "--no-check", "--json", "--commands", "0.1:screenshot:strict.png;0.2:quit"];
  expect(run(runCommand, false).code).toBe(0);
  const strictRun = run([...runCommand, "--require-checked"], false);
  expect(strictRun.code).toBe(1);
  expect(JSON.parse(strictRun.stdout.trim().split("\n").at(-1)!)).toMatchObject({ uncheckedCount: 1 });

  const uiCommand = [ogBin(), "ui", "check", game, "--screen", "start=", "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--settle", "0.1", "--json"];
  const ui = run(uiCommand, false);
  expect(ui.code).toBe(0);
  expect(JSON.parse(ui.stdout.trim())).toMatchObject({ uncheckedCount: 1 });
  const strictUi = run([...uiCommand, "--require-checked"], false);
  expect(strictUi.code).toBe(1);
  expect(JSON.parse(strictUi.stdout.trim())).toMatchObject({ uncheckedCount: 1 });

  writeFileSync(join(game, "tests/logic.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("simulation without GPU", async () => { expect((await runPlaytest({ commands: "0.2:quit" })).quit, "quit").toBe(true); });`);
  const logic = run([ogBin(), "test", game, "--filter", "logic.og.ts", "--json", "--no-check", "--require-checked"], false);
  if (logic.code !== 0) throw new Error(logic.out);
  expect(result(logic.stdout).ok).toBeTrue();
  writeFileSync(join(game, "tests/surface.og.ts"), SURFACE.replace('expect(run.quit, "reached quit").toBe(true)', 'expect(run.quit, "deliberate failure").toBe(false)'));
  const failed = run(command, false);
  expect(failed.code).toBe(1);
  expect(result(failed.stdout).status).toBe("failed");
  expect(result(failed.stdout).failures.join("; ")).toContain("deliberate failure");
  writeFileSync(join(game, "tests/surface.og.ts"), `import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";
test("frames only", async () => {
  const run = await runPlaytest({ commands: "0.1:screenshot:only.png;0.2:quit" });
  run.frame("only.png", () => expect(run.png("only.png").width, "width").toBeGreaterThan(0));
});`);
  const framesOnly = run(command, false);
  expect(framesOnly.code).toBe(0);
  // Its assertions are the frame reported unchecked: the test is not refused, and not passed.
  expect(result(framesOnly.stdout)).toMatchObject({ status: "unchecked", checks: 2, frameChecks: 0, unchecked: ["only.png"] });
}, 120_000);
