// `og polish` end to end on the native host: the topdown archetype's
// playtests are played headless and scored; the slime's bite (a damage fact
// with no hit stop) comes back as a finding mapped to og-feel §3, the hits
// with hit stop, shake, floating text and a sound score 100, every input is
// answered on its tick, and the report links frames that exist. A project
// without playtests is refused. The report stays in
// target/evidence/polish-test/ for inspection.
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
const evidence = join(repo, "target", "evidence", "polish-test");

const run = (cmd: string[]) => {
  const r = Bun.spawnSync(cmd, { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const decode = (b: Uint8Array) => new TextDecoder().decode(b);
  return { code: r.exitCode, out: decode(r.stdout) + decode(r.stderr) };
};

interface Polish {
  scores: Record<string, number | null>;
  findings: Array<{ measure: string; where: string; detail: string; fix: string }>;
  playtests: Array<{ playtest: string; damage: { moments: number; hitStop: number; floating: number; sound: number }; latency: { inputs: number; answered: number; medianTicks: number }; frame: { tier: string; frameP95Ms: number } }>;
}

test("og polish scores the topdown playtests and maps each finding to its skill", () => {
  const out = join(evidence, "topdown");
  rmSync(out, { recursive: true, force: true });
  const r = run([ogBin(), "polish", join(repo, "templates", "archetypes", "topdown"), "--out", out, "--frames", "3"]);
  if (r.code !== 0) throw new Error(r.out);
  const p = JSON.parse(readFileSync(join(out, "polish.json"), "utf8")) as Polish;
  expect(p.playtests.map((x) => x.playtest)).toEqual(["ai", "combat", "door", "flee", "frames", "replay", "walk"]);
  const combat = p.playtests.find((x) => x.playtest === "combat")!;
  expect(combat.damage.moments).toBeGreaterThan(2);
  expect(combat.damage.floating).toBe(combat.damage.moments);
  expect(combat.damage.sound).toBe(combat.damage.moments);
  expect(p.scores["hitFeedback"]).toBe(100);
  expect(p.scores["latency"]).toBe(100);
  for (const x of p.playtests) expect(x.latency.answered).toBe(x.latency.inputs);
  expect(p.scores["taste"]).toBeNull();
  // The slime's bite hurts the hero with no hit stop: a finding, mapped to og-feel §3.
  const bite = p.findings.find((f) => f.measure === "hit feedback" && f.where === "ai");
  expect(bite?.detail ?? "").toContain("no hit stop");
  expect(bite?.fix ?? "").toContain("og-feel §3");
  const report = readFileSync(join(out, "report.md"), "utf8");
  expect(report).toContain("| Hit feedback | 100 |");
  const frames = [...report.matchAll(/\]\((frames\/[^)]+\.png)\)/g)].map((m) => m[1]!);
  expect(frames.length).toBe(3);
  for (const f of frames) expect(existsSync(join(out, f))).toBeTrue();
}, 600_000);

test("og polish refuses a project without playtests", () => {
  const bare = join(evidence, "bare");
  rmSync(bare, { recursive: true, force: true });
  mkdirSync(join(bare, "src"), { recursive: true });
  writeFileSync(join(bare, "project.ogt"), 'type project\nversion 1\n---\nname "Bare"\nid "bare"\nengine "0.0.1"\nentry "src/main.ts"\n');
  writeFileSync(join(bare, "src", "main.ts"), 'import { og } from "@origingame/engine";\nexport default og.game({ tick: 60 });\n');
  const r = run([ogBin(), "polish", bare]);
  expect(r.code).toBe(1);
  expect(r.out).toContain("has no playtests");
  expect(existsSync(join(bare, "og-out", "polish", "report.md"))).toBeFalse();
});
