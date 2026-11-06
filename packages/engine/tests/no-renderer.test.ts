// Simulation does not need a GPU; a test that requests pixels does.
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";
import { attachRenderer } from "../src/node/index.ts";

declare const Bun: {
  spawnSync(cmd: string[], options: { cwd: string; env: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const evidence = join(repo, "target", "evidence", "no-renderer");
const game = join(evidence, "game");

const run = (cmd: string[], renderer: boolean) => {
  const env = { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: renderer ? undefined : "none" };
  const r = Bun.spawnSync(cmd, { cwd: repo, env });
  const stdout = new TextDecoder().decode(r.stdout);
  return { code: r.exitCode, stdout, out: stdout + new TextDecoder().decode(r.stderr) };
};

interface Result {
  playtest: string;
  ok: boolean;
  checks: number;
  frameChecks: number;
  unchecked: string[];
  failures: string[];
  tier: string;
  hash: string;
  hashes: string[];
}

const result = (stdout: string): Result => JSON.parse(stdout.split("\n").find((l) => l.startsWith('{"playtest"'))!);

const SURFACE = `import { playtest } from "@origingame/engine/playtest";
export default playtest({
  name: "a surface check needs a GPU without changing simulation",
  size: "640x480",
  runs: 2,
  commands: "0.3:dump_ui:ui.json;0.4:screenshot:surface.png;0.5:dump_world:world.json;0.6:quit",
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "no warnings");
    t.ok(run.quit, "reached quit");
    t.equal(run.hashes[0], run.hashes[1], "repeatable world");
    const ui = run.json("ui.json");
    t.equal([ui.surface.width, ui.surface.height], [640, 480], "requested surface");
    t.frame("surface.png", () => {
      const png = run.png("surface.png");
      t.equal([png.width, png.height], [640, 480], "rendered surface");
    });
  },
});
`;

test("missing GPU fails frame checks while simulation-only checks still pass", () => {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = run(["bun", join(repo, "packages/create/bin/create.ts"), game, "--name", "No Renderer"], true);
  if (created.code !== 0) throw new Error(created.out);
  writeFileSync(join(game, "tests/surface.og.ts"), SURFACE);
  const command = [ogBin(), "test", game, "--filter", "surface.og.ts", "--json", "--no-check"];
  const none = run(command, false);
  expect(none.code).not.toBe(0);
  const blind = result(none.stdout);
  expect(blind.ok).toBeFalse();
  expect(blind.tier).toBe("none");
  expect(blind.checks).toBe(4);
  expect(blind.frameChecks).toBe(0);
  expect(blind.unchecked).toEqual(["surface.png"]);
  expect(blind.failures.join("; ")).toContain("missing GPU renderer");
  const out = join(game, "og-out/test/surface");
  expect(existsSync(join(out, "surface.png"))).toBeFalse();
  const blindUi = readFileSync(join(out, "ui.json"), "utf8");

  const drawn = run(command, true);
  if (drawn.code !== 0) throw new Error(drawn.out);
  const lit = result(drawn.stdout);
  expect(lit.ok).toBeTrue();
  expect(lit.tier).not.toBe("none");
  expect(lit.hash).toBe(blind.hash);
  expect(lit.hashes).toEqual(blind.hashes);
  expect(lit.checks - lit.frameChecks).toBe(blind.checks);
  expect(lit.frameChecks).toBe(1);
  expect(lit.unchecked).toEqual([]);
  expect(existsSync(join(out, "surface.png"))).toBeTrue();
  expect(readFileSync(join(out, "ui.json"), "utf8")).toBe(blindUi);

  writeFileSync(join(game, "tests/logic.og.ts"), `import { playtest } from "@origingame/engine/playtest";
export default playtest({ name: "simulation without GPU", commands: "0.2:quit", check(run, t) { t.ok(run.quit, "quit"); } });`);
  const logic = run([ogBin(), "test", game, "--filter", "logic.og.ts", "--json", "--no-check"], false);
  if (logic.code !== 0) throw new Error(logic.out);
  expect(result(logic.stdout).ok).toBeTrue();
}, 120_000);

test("a refused adapter fails unless a renderer was explicitly disabled", () => {
  const reason = "no GPU adapter: Auto: no suitable adapter found";
  const refused = () => JSON.stringify({ ok: false, reason });
  const saved = process.env.OG_RENDERER;
  try {
    delete process.env.OG_RENDERER;
    expect(attachRenderer(1, 64, 64, refused)).toEqual({ ok: false, reason });
    process.env.OG_RENDERER = "none";
    expect(attachRenderer(1, 64, 64, refused)).toEqual({ ok: true, value: "none" });
  } finally {
    if (saved === undefined) delete process.env.OG_RENDERER;
    else process.env.OG_RENDERER = saved;
  }
});
