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
  // Every hit, the slime's bite on the hero too, holds the world: no hit-feedback finding.
  expect(combat.damage.hitStop).toBe(combat.damage.moments);
  expect(p.findings.filter((f) => f.measure === "hit feedback")).toEqual([]);
  const report = readFileSync(join(out, "report.md"), "utf8");
  expect(report).toContain("| Hit feedback | 100 |");
  expect(unresolved(report)).toEqual([]);
  const frames = [...report.matchAll(/\]\((frames\/[^)]+\.png)\)/g)].map((m) => m[1]!);
  expect(frames.length).toBe(3);
  for (const f of frames) expect(existsSync(join(out, f))).toBeTrue();
}, 600_000);

test("a damage moment without hit stop is a finding, mapped to og-feel §3", () => {
  const p = join(evidence, "no-hit-stop");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests"]) mkdirSync(join(p, d), { recursive: true });
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Hurt"\nid "hurt"\nengine "0.6.2"\nentry "src/main.ts"\n');
  // A hurt command lowers hp and shakes the camera, but nothing holds the world.
  writeFileSync(
    join(p, "src", "main.ts"),
    `import { og, defineResource, u32 } from "@origingame/engine";
const Life = defineResource("Life", { hp: u32(3) });
export default og.game({
  tick: 60,
  resources: [Life],
  commands: {
    hurt: (world) => {
      const life = world.resource(Life);
      if (life.ok) life.value.hp -= 1;
      og.camera.shake(world, 0.3);
    },
  },
  facts: (world) => {
    const life = world.resource(Life);
    return { hp: life.ok ? life.value.hp : 0 };
  },
});
`,
  );
  writeFileSync(join(p, "tests", "hurt.og.ts"), 'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "hurt", commands: "0.3:hurt;0.8:hurt;1.2:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n');
  const r = run([ogBin(), "polish", p, "--frames", "0"]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish;
  const f = out.findings.find((x) => x.measure === "hit feedback");
  expect(f?.detail ?? "").toContain("no hit stop");
  expect(f?.fix ?? "").toContain("og-feel §3");
  expect(unresolved(f?.fix ?? "")).toEqual([]);
}, 300_000);

test("an impact with three of the four signals owes no fourth; one with fewer is asked for each it lacks (rumpus-isle #13)", () => {
  const p = join(evidence, "three-of-four");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests", "assets/fonts"]) mkdirSync(join(p, d), { recursive: true });
  copyFileSync(join(repo, "templates", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf"), join(p, "assets", "fonts", "NotoSansSC-Regular-subset.ttf"));
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Bumps"\nid "bumps"\nengine "0.6.2"\nentry "src/main.ts"\n');
  // A bump (hits rising) holds the world, shakes and pops a number: three of
  // four, no sound. A fall (hp falling) only shakes: one of four.
  writeFileSync(
    join(p, "src", "main.ts"),
    `import { og, defineResource, u32 } from "@origingame/engine";
const Score = defineResource("Score", { hits: u32(0), hp: u32(3) });
export default og.game({
  tick: 60,
  resources: [Score],
  start: (world) => {
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: {
    bump: (world) => {
      const s = world.resource(Score);
      if (s.ok) s.value.hits += 1;
      og.time.hitStop(world, 4);
      og.camera.shake(world, 0.3);
      og.render.floatingText(world, { text: "bump", position: { x: 0, y: 1, z: 0 } });
    },
    fall: (world) => {
      const s = world.resource(Score);
      if (s.ok) s.value.hp -= 1;
      og.camera.shake(world, 0.3);
    },
  },
  facts: (world) => {
    const s = world.resource(Score);
    return s.ok ? { hits: s.value.hits, hp: s.value.hp } : {};
  },
});
`,
  );
  writeFileSync(join(p, "tests", "bumps.og.ts"), 'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "bumps", commands: "0.3:bump;0.8:fall;1.3:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n');
  const r = run([ogBin(), "polish", p, "--frames", "0"]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish;
  const dmg = out.playtests[0]!.damage as Polish["playtests"][number]["damage"] & { shake: number; whole: number };
  expect([dmg.moments, dmg.hitStop, dmg.shake, dmg.floating, dmg.sound, dmg.whole]).toEqual([2, 1, 2, 1, 0, 1]);
  const rows = out.findings.filter((x) => x.measure === "hit feedback").map((x) => x.detail.replace(/ \(.*/, ""));
  // The bump owes no sound; the fall owes hit stop, floating text and a sound, and not the shake it had.
  expect(rows).toEqual(["1 of 2 damage moments", "1 of 2 damage moments", "1 of 2 damage moments"]);
  const owed = out.findings.filter((x) => x.measure === "hit feedback").map((x) => /had no (.+?) within/.exec(x.detail)?.[1]);
  expect(owed).toEqual(["hit stop", "floating text", "a sound"]);
  // Per impact: three of three for the bump, one of three for the fall.
  expect(out.scores["hitFeedback"]).toBe(Math.round((100 * (3 + 1)) / 6));
}, 300_000);

test("og polish reads a dialog as a screen, leaves out what its scrim dims, and does not count touch controls as HUD", () => {
  const p = join(evidence, "screens");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests", "assets/fonts"]) mkdirSync(join(p, d), { recursive: true });
  copyFileSync(join(repo, "templates", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf"), join(p, "assets", "fonts", "NotoSansSC-Regular-subset.ttf"));
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Screens"\nid "screens"\nengine "0.6.2"\nentry "src/main.ts"\n');
  // Play: a score in a corner and the thumbs' controls (a stick and a big
  // button). Asking: a faint caption under a confirm dialog's scrim.
  writeFileSync(
    join(p, "src", "main.ts"),
    `import { og, defineResource, defineSystem, bool } from "@origingame/engine";
const Ask = defineResource("Ask", { open: bool(false), loading: bool(false) });
const Screen = defineSystem("Screen", { writes: [Ask] }, (_q, _t, world) => {
  const ask = world.resource(Ask);
  const open = ask.ok && ask.value.open;
  og.ui.anchor("Score", { anchor: "topLeft" }, () => og.ui.label("Points", "Score 120", { step: "body" }));
  og.ui.anchor("Thumbs", { anchor: "bottomLeft" }, () => og.ui.joystick("Stick", { size: 200 }));
  og.ui.anchor("Fire", { anchor: "bottomRight" }, () => og.ui.actionButton("Shoot", "Fire", { size: 160 }));
  og.ui.anchor("Note", { anchor: "center" }, () => og.ui.label("Faint", "Paused", { step: "body", color: { color: { r: 0.42, g: 0.42, b: 0.45 } } }));
  if (ask.ok && ask.value.loading) og.ui.loadingScreen("Load", { title: "Loading" });
  og.ui.confirmDialog("Quit", { open, title: "Leave the run?", message: "Your score is kept.", confirmLabel: "Leave" });
});
export default og.game({
  tick: 60,
  resources: [Ask],
  systems: [Screen],
  commands: {
    ask: (world) => { const a = world.resource(Ask); if (a.ok) a.value.open = true; },
    load: (world) => { const a = world.resource(Ask); if (a.ok) { a.value.open = false; a.value.loading = true; } },
  },
  start: (world) => {
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
});
`,
  );
  writeFileSync(
    join(p, "tests", "screens.og.ts"),
    'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "screens", size: "1280x720", commands: "0.5:screenshot:play.png;0.6:ask;1.4:screenshot:asking.png;1.5:load;2.3:screenshot:loading.png;2.4:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n',
  );
  const r = run([ogBin(), "polish", p, "--frames", "3"]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish & {
    playtests: Array<{ shots: Array<{ name: string; menu: boolean; hud: number; texts: number }>; contrast: { worst: Array<{ id: string }> } }>;
  };
  const [play, asking, loading] = out.playtests[0]!.shots;
  // Playing: a HUD, small although the stick and the button are big.
  expect(play!.menu).toBeFalse();
  expect(play!.hud).toBeLessThan(3);
  // Asking: the dialog is a screen, and the caption under its scrim is not read
  // (in play it is, and too faint).
  expect(asking!.menu).toBeTrue();
  expect(out.findings.some((f) => f.where.startsWith("screens play") && f.where.includes("Faint"))).toBeTrue();
  // The score, the caption and the action button's "Fire": a short word on a
  // wide control, read over its own glyphs (kestrel-stair FINDINGS #20), where
  // the 1 % farthest pixels of the button left it out as inkless.
  expect(play!.texts).toBe(3);
  expect(asking!.texts).toBe(4); // the title, the message, Leave and Cancel; not the score or the caption
  expect(out.findings.some((f) => f.where.startsWith("screens asking") && f.where.includes("Faint"))).toBeFalse();
  // Loading: one title in the centre, still a screen (its layer is a modal).
  expect(loading!.menu).toBeTrue();
  expect(out.findings.some((f) => f.measure === "HUD occupancy")).toBeFalse();
}, 300_000);

test("og polish reads a toast over a dialog at rest and leaves out one sliding out", () => {
  // lantern-vale FINDINGS #25: a toast leaving over the settings dialog read
  // 1.15:1 (the toasts' layer is over the scrim, but a fade's pixels are a
  // moment of it, not its colours). Two toasts over an open dialog: Saved
  // dismissed on the tick before the second shot, Kept at rest.
  const p = join(evidence, "toasts");
  rmSync(p, { recursive: true, force: true });
  for (const d of ["src", "tests", "assets/fonts"]) mkdirSync(join(p, d), { recursive: true });
  copyFileSync(join(repo, "templates", "game", "assets", "fonts", "NotoSansSC-Regular-subset.ttf"), join(p, "assets", "fonts", "NotoSansSC-Regular-subset.ttf"));
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Toasts"\nid "toasts"\nengine "0.6.2"\nentry "src/main.ts"\n');
  writeFileSync(
    join(p, "src", "main.ts"),
    `import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
const Shown = defineResource("Shown", { saved: u32(0) });
const Screen = defineSystem("Screen", { writes: [Shown] }, () => {
  og.ui.confirmDialog("Quit", { open: true, title: "Leave the run?", message: "Your score is kept.", confirmLabel: "Leave" });
});
export default og.game({
  tick: 60,
  resources: [Shown],
  systems: [Screen],
  commands: {
    toasts: (world) => {
      const s = world.resource(Shown);
      if (s.ok) s.value.saved = og.ui.toast(world, "Saved", { tone: "neutral" });
      og.ui.toast(world, "Kept", { tone: "neutral" });
    },
    dismiss: (world) => { const s = world.resource(Shown); if (s.ok) og.ui.dismissToast(world, s.value.saved); },
  },
  start: (world) => {
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
});
`,
  );
  writeFileSync(
    join(p, "tests", "toasts.og.ts"),
    'import { playtest } from "@origingame/engine/playtest";\nexport default playtest({ name: "toasts", size: "1280x720", commands: "0.2:toasts;1.0:screenshot:rest.png;1.1:dismiss;1.15:screenshot:leaving.png;1.3:quit", check(run, t) { t.ok(run.quit, "quit"); } });\n',
  );
  const r = run([ogBin(), "polish", p, "--frames", "2"]);
  if (r.code !== 0) throw new Error(r.out);
  const out = JSON.parse(readFileSync(join(p, "og-out", "polish", "polish.json"), "utf8")) as Polish & {
    playtests: Array<{ shots: Array<{ name: string; texts: number }> }>;
  };
  const [rest, leaving] = out.playtests[0]!.shots;
  // At rest: the dialog's title, message, Leave and Cancel, and both toasts
  // with their labels; leaving: Saved and its label are mid-exit.
  expect(rest!.texts).toBe(8);
  expect(leaving!.texts).toBe(6);
  expect(out.findings.some((f) => f.measure === "contrast" && f.where.includes("Saved"))).toBeFalse();
}, 300_000);

test("og polish refuses a project without playtests", () => {
  const bare = join(evidence, "bare");
  rmSync(bare, { recursive: true, force: true });
  mkdirSync(join(bare, "src"), { recursive: true });
  writeFileSync(join(bare, "project.ogt"), 'type project\nversion 1\n---\nname "Bare"\nid "bare"\nengine "0.6.2"\nentry "src/main.ts"\n');
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
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Locales"\nid "locales"\nengine "0.6.2"\nentry "src/main.ts"\n');
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
  writeFileSync(join(p, "project.ogt"), 'type project\nversion 1\n---\nname "Phone UI"\nid "phone-ui"\nengine "0.6.2"\nentry "src/main.ts"\n');
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
