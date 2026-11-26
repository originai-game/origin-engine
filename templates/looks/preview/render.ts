// Draws each look's preview (docs/art.md §2.6): the look's preset scene,
// cooked and run by the engine, so a preview never shows more than a game in
// that look draws. preview.json says which scene and art each look or
// variant takes; a scene is copied to target/look-previews/<name>/, its
// look set to extend the preset (and the variant), cooked and run headless
// for one frame at 1.5 s. The 2D scene is staged by 2d/stage.ts.
//
//   bun templates/looks/preview/render.ts [--out DIR] [--size WxH] [--webp] [--previews DIR] [--engine DIR] [--og FILE]
//                                         [--gpu] [--record FILE] [--slots FILE | name ...]
//
// A name is a look slug or <slug>:<variant>; none draws every look of the
// catalogue. Writes DIR/<name>.png (default target/look-previews/frames),
// 960x540 unless --size or the look's entry names a size; with --webp the
// catalogue's previews/<slug>.webp (a variant's previews/<slug>.<variant>.webp) too (960x540, by ImageMagick), into
// --previews (default packages/knowledge/previews). Run the og and addon of the commit under test (OG_NATIVE_ADDON,
// OG_ENGINE_PACKAGE): a preset is include_str! into og-cook. --engine is the
// engine package the scenes import and whose knowledge/looks/catalogue.json
// names the looks (default packages/engine, with packages/knowledge's
// catalogue). --gpu refuses any frame not drawn by a hardware adapter
// (OG_RENDERER=hardware, og run --require-checked, and the run's tier
// neither `none` nor a `-cpu` one); --record writes the run as JSON: the
// tier, the sha256 of og and the addon, the engine package's version, the
// checkout's commit and whether templates/looks/ is dirty there, and each
// preview's sha256 and size by its path under --previews (what scripts/release.sh
// previews records). --slots FILE draws the creation document's preview slots
// instead of names (create-game's previewSlots(), as JSON): a look or variant
// slot as above, a
// ui slot as the UI gallery's item card dressed in that pack (src/node/gallery.ts,
// 1280x720), each to --previews/<slot path>. Two ui
// slots drawn the same fail the run: a pack must change the frame.
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, symlinkSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { stage2d } from "./2d/stage.ts";
import type { Scene2d } from "./2d/stage.ts";

const repo = resolve(import.meta.dir, "../../..");
const looks = join(repo, "templates/looks");
const args = process.argv.slice(2);
const flag = (k: string, d: string) => {
  const i = args.indexOf(k);
  if (i < 0) return d;
  const v = args[i + 1]!;
  args.splice(i, 2);
  return v;
};
const out = resolve(flag("--out", join(repo, "target/look-previews/frames")));
const size = flag("--size", "");
const og = resolve(flag("--og", join(repo, "target/debug/og")));
// --previews is the directory a slot's `previews/...` path lands in.
const previews = resolve(flag("--previews", join(repo, "packages/knowledge/previews")));
const engine = resolve(flag("--engine", join(repo, "packages/engine")));
const record = flag("--record", "");
const webp = args.includes("--webp");
const gpu = args.includes("--gpu");
const slotsFile = flag("--slots", "");
const names = args.filter((a) => !a.startsWith("--"));
/** One preview to draw: create-game's PreviewSlot (bin/creation.ts). */
interface Slot {
  path: string;
  kind: "look" | "variant" | "ui";
  look?: string;
  variant?: string;
  pack?: string;
}

interface Scene extends Scene2d {
  scene: "2d" | "3d";
  models?: string;
  sprites?: Record<string, { file: string; height: number }>;
  /** The frame's size when --size names none: a pixel look's whole multiple of its grid. */
  size?: string;
}
const table = JSON.parse(readFileSync(join(import.meta.dir, "preview.json"), "utf8")) as Record<string, Scene>;
const packed = join(engine, "knowledge/looks/catalogue.json");
const catalogue = JSON.parse(readFileSync(existsSync(packed) ? packed : join(repo, "packages/knowledge/looks/catalogue.json"), "utf8")) as { styles: { slug: string }[] };
const slots: Slot[] = slotsFile
  ? (JSON.parse(readFileSync(slotsFile, "utf8")) as Slot[])
  : (names.length ? names : catalogue.styles.map((s) => s.slug)).map((n) => {
      const [look, variant] = n.split(":") as [string, string | undefined];
      return { path: `previews/${variant ? `${look}.${variant}` : look}.webp`, kind: variant ? "variant" : "look", look, variant };
    });

/** An image's width and height, from its PNG or JPEG header. */
function size2(file: string): [number, number] {
  const b = readFileSync(file);
  if (b.readUInt32BE(0) === 0x89504e47) return [b.readUInt32BE(16), b.readUInt32BE(20)];
  for (let i = 2; i < b.length; ) {
    const marker = b[i + 1]!;
    if (marker >= 0xc0 && marker <= 0xc3) return [b.readUInt16BE(i + 7), b.readUInt16BE(i + 5)];
    i += 2 + b.readUInt16BE(i + 2);
  }
  throw new Error(`${file}: no size in its header`);
}
const aspect = (file: string) => {
  const [w, h] = size2(file);
  return w / h;
};

mkdirSync(out, { recursive: true });
if (webp) mkdirSync(previews, { recursive: true });
// ImageMagick 7's `magick`, else 6's `convert` (the GPU host has only 6), with the same arguments.
const magick = spawnSync("magick", ["-version"]).status === 0 ? "magick" : "convert";
let failed = 0;
const tiers = new Set<string>();
/** The catalogue's 960x540 webp of a frame (with --webp; true without). */
function toWebp(frame: string, preview: string): boolean {
  if (!webp) return true;
  mkdirSync(join(preview, ".."), { recursive: true });
  const m = spawnSync(magick, [frame, "-resize", "960x540^", "-gravity", "center", "-extent", "960x540", "-quality", "82", "-define", "webp:method=6", preview], { encoding: "utf8" });
  if (m.status !== 0) {
    console.log(`${preview}: magick failed: ${m.error?.message ?? m.stderr.trim()}`);
    return false;
  }
  console.log(`  ${preview}`);
  return true;
}
const env = gpu ? { ...process.env, OG_RENDERER: "hardware" } : process.env;
/** Draws a ui slot: the UI gallery's item card (src/node/gallery.ts) in the slot's pack, at 1280x720. */
function drawUi(slot: Slot, file: string): string | null {
  const at = join(repo, "target/look-previews", `ui@${slot.pack}`);
  rmSync(at, { recursive: true, force: true });
  const run = spawnSync("bun", [join(engine, "src/node/gallery.ts"), "--only", "panels/item-card · default", "--packs", slot.pack!, "--out", at, "--og", og], { encoding: "utf8", timeout: 600_000, env: { ...env, OG_ENGINE_PACKAGE: engine, OG_REPO: repo } });
  const frame = join(at, slot.pack!, "0000.png");
  if (run.status !== 0 || !existsSync(frame)) {
    console.log(`${slot.path}: the gallery failed (${run.status}): ${(run.stderr || run.stdout).trim().split("\n").slice(-4).join(" | ")}`);
    return null;
  }
  cpSync(frame, join(out, file));
  // OG_RENDERER=hardware refused any software adapter before the frame was drawn.
  if (gpu) tiers.add("vulkan");
  return join(out, file);
}

for (const slot of slots) {
  const slug = slot.look ?? "";
  const variant = slot.variant;
  const name = slot.kind === "ui" ? `ui@${slot.pack}` : variant ? `${slug}:${variant}` : slug;
  const preview = join(previews, slot.path.replace(/^previews\//, ""));
  if (slot.kind === "ui") {
    const file = `${name}.png`;
    const frame = drawUi(slot, file);
    if (!frame || !toWebp(frame, preview)) failed++;
    continue;
  }
  const scene: Scene = table[name] ?? table[slug] ?? { scene: "3d" };
  const dir = join(repo, "target/look-previews", name.replace(":", "@"));
  rmSync(dir, { recursive: true, force: true });
  cpSync(join(import.meta.dir, scene.scene), dir, { recursive: true, filter: (from) => !/\/2d\/(tools|stage\.ts)/.test(from) });
  mkdirSync(join(dir, "node_modules/@origingame"), { recursive: true });
  symlinkSync(engine, join(dir, "node_modules/@origingame/engine"));
  mkdirSync(join(dir, "assets/looks"), { recursive: true });
  writeFileSync(join(dir, "assets/looks/main.ogt"), `type look\nversion 1\n---\nname "Preview: ${name}"\nextends ${JSON.stringify(slug)}\n${variant ? `variant ${JSON.stringify(variant)}\n` : ""}`);
  if (scene.models) cpSync(join(looks, scene.models), join(dir, "assets/models"), { recursive: true });
  if (scene.scene === "3d") {
    const sprites: Record<string, { path: string; width: number; height: number }> = {};
    if (scene.art) cpSync(join(looks, scene.art), join(dir, "assets/art"), { recursive: true });
    for (const [beat, s] of Object.entries(scene.sprites ?? {})) {
      sprites[beat] = { path: `art/${s.file}`, width: s.height * aspect(join(looks, scene.art!, s.file)), height: s.height };
    }
    const ground = existsSync(join(dir, "assets/models/field.ogt")) ? "models/field.ogt" : "models/ground.glb";
    writeFileSync(join(dir, "src/sprites.ts"), `// Written by templates/looks/preview/render.ts from preview.json.\nexport const GROUND = ${JSON.stringify(ground)};\nexport const SPRITES: Record<string, { path: string; width: number; height: number }> = ${JSON.stringify(sprites)};\n`);
  } else {
    stage2d(dir, name, scene, looks, repo);
  }
  const file = `${name.replace(":", "@")}.png`;
  rmSync(join(out, file), { force: true });
  const run = spawnSync(og, ["run", dir, "--headless", "--size", size || scene.size || "960x540", "--out", out, "--commands", `1.5:screenshot:${file};1.6:quit`, ...(gpu ? ["--json", "--require-checked"] : [])], {
    encoding: "utf8",
    timeout: 300_000,
    env,
  });
  if (run.status !== 0 || !existsSync(join(out, file))) {
    failed++;
    console.log(`${name}: run failed (${run.status}): ${(run.stderr || run.stdout).trim().split("\n").slice(-4).join(" | ")}`);
    continue;
  }
  if (gpu) {
    const summary = run.stdout.split("\n").reverse().find((l) => l.startsWith("{"));
    const tier = (summary && (JSON.parse(summary) as { tier?: string }).tier) || "none";
    if (tier === "none" || tier.endsWith("-cpu")) {
      failed++;
      console.log(`${name}: drawn on ${tier}, not a hardware adapter (--gpu)`);
      continue;
    }
    tiers.add(tier);
  }
  // The --json summary line names its own `warnings` list: only prose lines are warnings.
  const warnings = (run.stdout + run.stderr).split("\n").filter((l) => /warning/.test(l) && !l.startsWith("{"));
  console.log(`${name}: ${join(out, file)}${warnings.length ? `\n  ${warnings.join("\n  ")}` : ""}`);
  if (webp && !toWebp(join(out, file), preview)) failed++;
}
// A pack that leaves the frame as another pack's drew nothing of its own.
const uiDrawn = new Map<string, string>();
for (const slot of webp ? slots.filter((s) => s.kind === "ui") : []) {
  const at = join(previews, slot.path.replace(/^previews\//, ""));
  if (!existsSync(at)) continue;
  const digest = createHash("sha256").update(readFileSync(at)).digest("hex");
  const same = uiDrawn.get(digest);
  if (same) {
    failed++;
    console.log(`${slot.path}: the same frame as ${same}: the pack did not dress the page`);
  } else uiDrawn.set(digest, slot.path);
}
if (record && !failed) {
  const sha256 = (f: string) => createHash("sha256").update(readFileSync(f)).digest("hex");
  const git = (...a: string[]) => execFileSync("git", a, { cwd: repo, encoding: "utf8" }).trim();
  const addon = process.env.OG_NATIVE_ADDON;
  const paths = webp ? slots.map((s) => s.path) : [];
  const run = {
    engine: (JSON.parse(readFileSync(join(engine, "package.json"), "utf8")) as { version: string }).version,
    revision: git("rev-parse", "HEAD"),
    dirty: git("status", "--porcelain", "--", "templates/looks") !== "",
    tier: [...tiers].sort().join(",") || null,
    og: sha256(og),
    addon: addon ? sha256(addon) : null,
    previews: Object.fromEntries(paths.map((p) => [p, { sha256: sha256(join(previews, p.replace(/^previews\//, ""))), size: statSync(join(previews, p.replace(/^previews\//, ""))).size }])),
  };
  writeFileSync(record, `${JSON.stringify(run, null, 2)}\n`);
  console.log(`record: ${record}`);
}
process.exit(failed ? 1 : 0);
