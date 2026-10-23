// A game without a renderer (OG_RENDERER=none: the platform's engine and
// probe sandboxes carry none, docs/design.md §5) on the native host: the
// scaffolded game template, with one more playtest at a 640x480 surface,
// runs `og test` with the switch on and off.
//
// Without a renderer: og test exits 0, each screenshot is one
// "no renderer, frame unchecked" line and the summary counts them, no PNG
// is written, every check outside t.frame runs (with a renderer, the same
// count plus the frame checks) and the UI lays out on the playtest's own
// surface. Both ways land on the same world hashes and the same dumps:
// rendering never feeds back into simulation. `og run --json` says the
// tier is `none` and lists the unchecked frame; `og polish` says "no
// renderer" in its frame-time rows.
//
// Ways this could fail (each did before the change): a screenshot without
// a renderer warned, so "the script ran without warnings" failed; the
// smoke playtest's run.png threw and failed it; og test exited 1; the UI
// laid out on 1280x720 whatever the playtest's size; og polish scored a
// frame time from the step alone. The scaffold stays in
// target/evidence/no-renderer/ for inspection.
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
const evidence = join(repo, "target", "evidence", "no-renderer");
const game = join(evidence, "game");

const run = (cmd: string[], renderer: boolean) => {
  const env: Record<string, string | undefined> = { ...process.env, OG_ENGINE_PACKAGE: pkg, OG_RENDERER: renderer ? undefined : "none" };
  const r = Bun.spawnSync(cmd, { cwd: repo, env });
  const decode = (b: Uint8Array) => new TextDecoder().decode(b);
  return { code: r.exitCode, stdout: decode(r.stdout), out: decode(r.stdout) + decode(r.stderr) };
};

interface Result {
  playtest: string;
  ok: boolean;
  checks: number;
  frameChecks: number;
  unchecked: string[];
  failures: string[];
  notes: string[];
  tier: string;
  hash: string;
  hashes: string[];
  warnings: string[];
}

const results = (stdout: string): Map<string, Result> =>
  new Map(
    stdout
      .split("\n")
      .filter((l) => l.startsWith('{"playtest"'))
      .map((l) => JSON.parse(l) as Result)
      .map((r) => [r.playtest, r]),
  );

// The playtest the template lacks: a non-default surface, its UI dumped
// (checked without a renderer too) and its frame read (only with one).
const SURFACE = `import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "the UI lays out on the playtest's 640x480 surface, renderer or not",
  size: "640x480",
  runs: 2,
  commands: "0.3:dump_ui:ui.json;0.4:screenshot:surface.png;0.5:dump_world:world.json;0.6:quit",
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(run.hashes[0], run.hashes[1], "two runs land on the same hash");
    const ui = run.json<{ surface: { width: number; height: number } }>("ui.json");
    t.equal([ui.surface.width, ui.surface.height], [640, 480], "the UI's surface is the playtest's size");
    t.frame("surface.png", () => {
      const png = run.png("surface.png");
      t.equal([png.width, png.height], [640, 480], "the frame is 640x480");
    });
  },
});
`;

/** A dump with the systems' wall-clock timings left out. */
const dump = (dir: string, file: string): unknown => {
  const v = JSON.parse(readFileSync(join(dir, file), "utf8")) as { systems?: Array<Record<string, unknown>>; render?: unknown };
  for (const s of v.systems ?? []) {
    delete s["totalMs"];
    delete s["ms"];
  }
  // An entity's `render` block (the level of detail drawn) is what the
  // renderer did, not the game: without one there is nothing to report.
  delete v.render;
  return v;
};

test("og test without a renderer passes, counts its unchecked frames and runs the same game", () => {
  rmSync(evidence, { recursive: true, force: true });
  mkdirSync(evidence, { recursive: true });
  const created = run(["bun", join(repo, "packages", "create", "bin", "create.ts"), game, "--name", "No Renderer"], true);
  if (created.code !== 0) throw new Error(created.out);
  writeFileSync(join(game, "tests", "surface.og.ts"), SURFACE);

  // Without a renderer, as a person reads it: a line per frame and the count.
  const text = run([ogBin(), "test", game], false);
  if (text.code !== 0) throw new Error(text.out);
  expect(text.stdout).toMatch(/PASS smoke \(\d+ checks, 2 frames unchecked/);
  // The region expectation reads the frame too: unchecked, like the screenshot.
  expect(text.stdout).toContain("  ~ og run: expect:region:560,16,64,24:color:#9da5b1:24: no renderer, frame unchecked\n");
  expect(text.stdout).toContain("  ~ smoke.png: no renderer, frame unchecked\n");
  expect(text.stdout).toContain("  ~ surface.png: no renderer, frame unchecked\n");
  // smoke (the region expectation and its screenshot), guard, intro (two frames) and the surface suite written above.
  expect(text.stdout).toMatch(/PASS intro \(\d+ checks, 2 frames unchecked/);
  expect(text.stdout).toContain("og test: 4 suite(s) passed; 5 frame(s) unchecked: no renderer here");
  expect(existsSync(join(game, "og-out", "test", "smoke", "smoke.png"))).toBeFalse();

  // And as JSON, with the dumps kept for the comparison below.
  const none = run([ogBin(), "test", game, "--json"], false);
  if (none.code !== 0) throw new Error(none.out);
  const blind = results(none.stdout);
  expect([...blind.keys()].sort()).toEqual(["guard", "intro", "smoke", "surface"]);
  for (const r of blind.values()) {
    expect(r.tier).toBe("none");
    expect(r.failures).toEqual([]);
    expect(r.warnings).toEqual([]);
    expect(r.frameChecks).toBe(0);
    expect(r.notes).toEqual([]);
  }
  expect(blind.get("smoke")!.unchecked).toEqual(["og run: expect:region:560,16,64,24:color:#9da5b1:24", "smoke.png"]);
  expect(blind.get("surface")!.unchecked).toEqual(["surface.png"]);
  expect(blind.get("guard")!.unchecked).toEqual([]);
  expect(blind.get("intro")!.unchecked).toEqual(["intro-4.png", "intro-8.png"]);
  const smokeOut = join(game, "og-out", "test", "smoke");
  const surfaceOut = join(game, "og-out", "test", "surface");
  const blindDumps = {
    smoke: ["world.json", "ui.json", "trickster.json", "crate.json"].map((f) => dump(smokeOut, f)),
    surface: ["world.json", "ui.json"].map((f) => dump(surfaceOut, f)),
  };

  // With a renderer (where this machine has one): the same hashes and dumps,
  // and the same checks plus the frame checks.
  const lit = results(run([ogBin(), "test", game, "--json"], true).stdout);
  expect([...lit.keys()].sort()).toEqual(["guard", "intro", "smoke", "surface"]);
  for (const [name, r] of lit) {
    const b = blind.get(name)!;
    expect(r.failures).toEqual([]);
    expect({ name, hash: r.hash, hashes: r.hashes }).toEqual({ name, hash: b.hash, hashes: b.hashes });
    expect({ name, checks: r.checks - r.frameChecks }).toEqual({ name, checks: b.checks });
    if (r.tier !== "none") expect({ name, unchecked: r.unchecked }).toEqual({ name, unchecked: [] });
  }
  if (lit.get("smoke")!.tier !== "none") {
    expect(lit.get("smoke")!.frameChecks).toBe(1);
    expect(lit.get("surface")!.frameChecks).toBe(1);
    expect(existsSync(join(smokeOut, "smoke.png"))).toBeTrue();
  }
  expect(["world.json", "ui.json", "trickster.json", "crate.json"].map((f) => dump(smokeOut, f))).toEqual(blindDumps.smoke);
  expect(["world.json", "ui.json"].map((f) => dump(surfaceOut, f))).toEqual(blindDumps.surface);

  // og run --json: the tier and the frame it did not take.
  const out = join(evidence, "run");
  const r = run([ogBin(), "run", "--headless", game, "--out", out, "--commands", "0.2:screenshot:a.png;0.3:dump_world:w.json;0.4:quit", "--json"], false);
  if (r.code !== 0) throw new Error(r.out);
  const summary = JSON.parse(r.stdout.trim().split("\n").pop()!) as { tier: string; unchecked: string[]; warnings: string[]; outputs: string[] };
  expect(summary.tier).toBe("none");
  expect(summary.unchecked).toEqual(["a.png"]);
  expect(summary.warnings).toEqual([]);
  expect(summary.outputs).toEqual([join(out, "w.json")]);
  expect(r.out).toMatch(/INFO tick \d+: 0\.2s screenshot: no renderer, frame unchecked \(a\.png\)/);

  // og polish: frame time is not measured without a renderer, and says so.
  const polished = join(evidence, "polish");
  const p = run([ogBin(), "polish", game, "--out", polished], false);
  if (p.code !== 0) throw new Error(p.out);
  const report = readFileSync(join(polished, "report.md"), "utf8");
  expect(report).toContain("| Frame time | n/a (no renderer) |");
  expect(report).toMatch(/\| Frame \(no renderer\) \| step [\d.]+ ms median, [\d.]+ p95; no renderer: render and frame time not measured; 1 frame\(s\) unchecked \|/);
  const polish = JSON.parse(readFileSync(join(polished, "polish.json"), "utf8")) as { scores: Record<string, number | null>; findings: Array<{ measure: string }> };
  expect(polish.scores["frameTime"]).toBeNull();
  expect(polish.findings.filter((f) => f.measure === "frame time")).toEqual([]);
}, 600_000);
