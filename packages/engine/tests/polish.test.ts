// `og polish` end to end on the native host: the topdown archetype's
// playtests are played headless and scored; the slime's bite (a damage fact
// with no hit stop) comes back as a finding mapped to og-feel §3, the hits
// with hit stop, shake, floating text and a sound score 100, every input is
// answered on its tick, and the report links frames that exist. A project
// without playtests is refused. The report stays in
// target/evidence/polish-test/ for inspection.
import { expect, test } from "bun:test";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

/** The skill sections a text names (`og-feel §3`, `og-taste §4, §10`) that the knowledge package's skills do not have. */
function unresolved(text: string): string[] {
  const missing: string[] = [];
  for (const m of text.matchAll(/(og-[a-z]+(?:-[a-z]+)*) §(\d+)((?:, §\d+| and §\d+)*)/g)) {
    const skill = join(repo, "packages", "knowledge", "skills", m[1]!, "SKILL.md");
    const have = existsSync(skill) ? new Set([...readFileSync(skill, "utf8").matchAll(/^## (\d+)\. /gm)].map((h) => h[1]!)) : new Set<string>();
    for (const n of [m[2]!, ...[...m[3]!.matchAll(/§(\d+)/g)].map((x) => x[1]!)]) if (!have.has(n)) missing.push(`${m[1]} §${n}`);
  }
  return missing;
}

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
  expect(unresolved(report)).toEqual([]);
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

test("og polish reports the string tables' missing keys as a finding for og-i18n", () => {
  const p = join(evidence, "locales");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests", "assets/strings"]) mkdirSync(join(p, d), { recursive: true });
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Locales"\nid "locales"\nengine "0.0.1"\nentry "src/main.ts"\n');
  writeFileSync(join(p, "src", "main.ts"), 'import { og } from "@origingame/engine";\nexport default og.game({ tick: 60, start: (world) => { og.i18n.load(world, "strings/en.ogt"); og.i18n.load(world, "strings/ja.ogt"); } });\n');
  writeFileSync(join(p, "assets", "strings", "en.ogt"), 'type strings\nversion 1\n---\nlocale "en"\nstring "menu.play" "Play"\nstring "menu.quit" "Quit"\n');
  writeFileSync(join(p, "assets", "strings", "ja.ogt"), 'type strings\nversion 1\n---\nlocale "ja"\nstring "menu.play" "プレイ"\n');
  writeFileSync(join(p, "tests", "idle.og.ts"), 'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "idle", commands: "0.2:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n');
  const r = run([ogBin(), "polish", p]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish & { warnings: string[] };
  expect(out.warnings).toEqual(["assets/strings/ja.ogt: missing key 'menu.quit' (en has it)"]);
  expect(out.scores["locale"]).toBeLessThan(100);
  const f = out.findings.find((x) => x.measure === "locale keys");
  expect(f?.fix ?? "").toContain("og-i18n");
}, 300_000);

test("og polish reads a phone's UI at its device pixels and measures text, not disabled controls", () => {
  const p = join(evidence, "phone-ui");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests", "assets/fonts"]) mkdirSync(join(p, d), { recursive: true });
  copyFileSync(join(repo, "templates", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf"), join(p, "assets", "fonts", "NotoSansSC-Regular-subset.ttf"));
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Phone UI"\nid "phone-ui"\nengine "0.0.1"\nentry "src/main.ts"\n');
  // A phone at 3x: a faint label (a real finding), a readable one, and a
  // disabled button whose dimmed text is exempt.
  writeFileSync(
    join(p, "src", "main.ts"),
    `import { og, defineSystem } from "@origingame/engine";
const Screen = defineSystem("Screen", {}, () => {
  og.ui.panel("Menu", { width: "fill" }, () => {
    og.ui.label("Faint", "Hard to read", { step: "body", color: { color: { r: 0.17, g: 0.17, b: 0.19 } } });
    og.ui.label("Plain", "Easy to read", { step: "body" });
    og.ui.button("Locked", "Locked door", { disabled: true });
  });
});
export default og.game({
  tick: 60,
  systems: [Screen],
  start: (world) => {
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
    og.ui.previewViewport(world, { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" });
  },
});
`,
  );
  writeFileSync(join(p, "tests", "menu.og.ts"), 'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "menu", size: "1170x2532", commands: "0.5:screenshot:menu.png;0.6:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n');
  const r = run([ogBin(), "polish", p, "--frames", "1"]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish & {
    playtests: Array<{ contrast: { texts: number; passing: number; worst: Array<{ id: string; ratio: number }> } }>;
  };
  const c = out.playtests[0]!.contrast;
  // Two labels measured where they are drawn: the plain one passes, the faint one does not; the disabled button is not counted.
  expect(c.texts).toBe(2);
  expect(c.passing).toBe(1);
  expect(c.worst[0]!.id).toBe("Faint");
  expect(out.findings.some((f) => f.where.includes("Locked"))).toBeFalse();
  expect(out.findings.some((f) => f.measure === "contrast" && f.where.includes("Faint"))).toBeTrue();
}, 300_000);
