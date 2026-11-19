// Draws a showcase hero scene in every 3D look and variant: the game as it
// is (its set, camera, light and code), with only its look swapped. The
// project is copied to target/look-previews/heroes/<hero>/<name>/ and the
// copy's look file is the game's own with its `extends` line naming the
// preset (and a `variant` line after it), so the game's mood, exposure and
// grade lie over every preset, as for a game that changed its look; nothing
// under showcase/ is written. The game's look unchanged is the frame named
// `game`.
//
//   bun templates/looks/preview/heroes.ts <hero> [--size WxH] [--settle SECONDS] [--out DIR] [--sheet] [--no-draw] [--keep] [name ...]
//
// <hero> is a project under showcase/ listed in HEROES. A name is a look
// slug, <slug>:<variant> or `game`; none draws every 3D look of the
// catalogue and each of its variants. Writes DIR/<name>.png (default
// target/look-previews/heroes/<hero>/frames); --sheet also writes
// DIR/sheet.jpg, a labelled contact sheet of every frame in DIR (with
// --no-draw, of the frames already there, drawing none); --keep reuses a
// copy already there (and its cook). --settle is how long the
// frame is drawn before it is taken (default 1 s; a software renderer
// draws these scenes at many seconds a frame, so give it 0.1). Run the og
// and addon of the commit under test (OG_ENGINE_PACKAGE, OG_NATIVE_ADDON).
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const repo = resolve(import.meta.dir, "../../..");

/**
 * A hero's frame: the commands that set it up at tick 0, and the seconds
 * the scene runs before the frame. Those seconds are stepped without
 * drawing (`advance`); the frames drawn are the --settle seconds after
 * them, for the temporal passes and the exposure.
 */
interface Hero {
  commands: string;
  hold: number;
}
const HEROES: Record<string, Hero> = {
  // The cover as the game starts (sunset, clear, summer), once the cloak
  // hangs and the sea runs.
  "the-last-light": { commands: "", hold: 4 },
  // The HUD off (the UI is never stylised); rain on a spring night, held
  // until the street has soaked and its puddles stand (tools/capture.sh).
  "night-shift": { commands: "0:hud", hold: 20 },
};

const args = process.argv.slice(2);
const flag = (k: string, d: string) => {
  const i = args.indexOf(k);
  if (i < 0) return d;
  const v = args[i + 1]!;
  args.splice(i, 2);
  return v;
};
const size = flag("--size", "960x540");
const settle = Number(flag("--settle", "1"));
const og = resolve(flag("--og", join(repo, "target/debug/og")));
const outFlag = flag("--out", "");
const sheet = args.includes("--sheet");
const keep = args.includes("--keep");
const draw = !args.includes("--no-draw");
const [hero, ...names] = args.filter((a) => !a.startsWith("--"));
const setup = hero ? HEROES[hero] : undefined;
if (!hero || !setup) {
  console.error(`heroes.ts: name a hero scene: ${Object.keys(HEROES).join(", ")}`);
  process.exit(2);
}
const project = join(repo, "showcase", hero);
const work = join(repo, "target/look-previews/heroes", hero);
const out = resolve(outFlag || join(work, "frames"));

interface Style {
  slug: string;
  look: { profiles: string[] };
  variants?: { name: string }[];
}
const catalogue = JSON.parse(readFileSync(join(repo, "packages/knowledge/looks/catalogue.json"), "utf8")) as { styles: Style[] };
/** Every 3D look, each followed by its variants, in the catalogue's order. */
const all = catalogue.styles.filter((s) => s.look.profiles.includes("3d")).flatMap((s) => [s.slug, ...(s.variants ?? []).map((v) => `${s.slug}:${v.name}`)]);
const todo = !draw ? [] : names.length ? names : all;

const look = /^look\s+"([^"]+)"/m.exec(readFileSync(join(project, "project.ogt"), "utf8"))?.[1];
if (!look) throw new Error(`${project}/project.ogt names no look`);
const ownLook = readFileSync(join(project, "assets", look), "utf8");
if (!/^extends\s+"[^"]+"\s*$/m.test(ownLook)) throw new Error(`${project}/assets/${look} extends no preset`);

/** The game's look on another preset: its `extends` line names the preset and the variant follows it; its mood, ui and overrides stay. */
function swapped(own: string, slug: string, variant: string | undefined): string {
  const preset = `extends ${JSON.stringify(slug)}${variant ? `\nvariant ${JSON.stringify(variant)}` : ""}`;
  return own
    .split("\n")
    .filter((line) => !/^variant\s/.test(line))
    .map((line) => (/^extends\s/.test(line) ? preset : line))
    .join("\n");
}

const ensure = spawnSync("bun", [join(repo, "scripts/assets.ts"), "ensure", project], { encoding: "utf8" });
if (ensure.status !== 0) {
  console.error((ensure.stderr || ensure.stdout).trim());
  process.exit(1);
}

/** What a copy leaves behind: outputs, installs and the tools that made the assets. */
const SKIP = new Set(["cooked", "dist", "og-out", ".og", "node_modules", "tools", "tests"]);
const fileOf = (name: string) => `${name.replace(":", "@")}.png`;

mkdirSync(out, { recursive: true });
let failed = 0;
for (const name of todo) {
  const [slug, variant] = name.split(":") as [string, string | undefined];
  const dir = join(work, name.replace(":", "@"));
  if (!keep || !existsSync(dir)) {
    rmSync(dir, { recursive: true, force: true });
    cpSync(project, dir, { recursive: true, filter: (src) => !SKIP.has(basename(src)) });
    mkdirSync(join(dir, "node_modules/@origingame"), { recursive: true });
    symlinkSync(join(repo, "packages/engine"), join(dir, "node_modules/@origingame/engine"));
  }
  writeFileSync(join(dir, "assets", look), name === "game" ? ownLook : swapped(ownLook, slug, variant));
  const file = fileOf(name);
  rmSync(join(out, file), { force: true });
  const at = setup.hold + settle;
  const commands = `${setup.commands ? `${setup.commands};` : ""}0:advance:${setup.hold * 60};${at}:screenshot:${file};${at + 0.05}:quit`;
  const started = Date.now();
  const run = spawnSync(og, ["run", dir, "--headless", "--size", size, "--out", out, "--commands", commands], { encoding: "utf8", timeout: 3_600_000, maxBuffer: 1 << 28 });
  const log = run.stdout + run.stderr;
  writeFileSync(join(work, `${name.replace(":", "@")}.log`), log);
  const seconds = Math.round((Date.now() - started) / 1000);
  if (run.status !== 0 || !existsSync(join(out, file))) {
    failed++;
    console.log(`${name}: run failed (${run.status}) after ${seconds} s: ${log.trim().split("\n").slice(-4).join(" | ")}`);
    continue;
  }
  const warnings = log.split("\n").filter((l) => /warning/.test(l));
  console.log(`${name}: ${join(out, file)} (${seconds} s)${warnings.length ? `\n  ${warnings.join("\n  ")}` : ""}`);
}

if (sheet) {
  // The game's own frame first, then the looks in the catalogue's order.
  const frames = ["game", ...all].filter((n) => existsSync(join(out, fileOf(n))));
  const tiles = frames.flatMap((n) => ["-label", n, join(out, fileOf(n))]);
  const m = spawnSync("magick", ["montage", ...tiles, "-tile", "6x", "-geometry", "480x270+6+6", "-background", "#16181c", "-fill", "#e8e8e8", "-pointsize", "17", "-quality", "90", join(out, "sheet.jpg")], { encoding: "utf8" });
  if (m.status !== 0) {
    failed++;
    console.log(`sheet: magick failed: ${m.stderr.trim()}`);
  } else console.log(`sheet: ${join(out, "sheet.jpg")} (${frames.length} frames)`);
}
process.exit(failed ? 1 : 0);
